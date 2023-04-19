use screen::{Message, ReturnMessage};

use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc::channel, Mutex};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use winit::event::{Event, WindowEvent};

struct State {
    windows: HashMap<usize, Window>,
}

struct Window {
    window: winit::window::Window,
    pixels: pixels::Pixels,
}

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let state = Arc::new(Mutex::new(State {
        windows: HashMap::new(),
    }));
    let async_state = state.clone();
    let (event_send, mut event_recv) = channel(2);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("failed to build runtime");
    runtime.spawn(async move {
        let state = async_state;

        let socket_addr = std::env::args().nth(1).expect("socket addr not provided");
        let socket = interprocess::local_socket::tokio::LocalSocketStream::connect(socket_addr)
            .await
            .expect("failed to connect to socket");
        let (reader, writer) = socket.into_split();
        let mut reader = BufReader::new(reader.compat());
        let mut writer = writer.compat_write();

        tokio::task::spawn(async move {
            let mut buf = String::with_capacity(4096);
            loop {
                if let Err(e) = reader.read_line(&mut buf).await {
                    eprintln!("error reading socket buffer: {e:?}")
                }
                let Ok(message) = ron::from_str::<Message>(&buf) else {
                eprintln!("error reading message");

                continue;
            };
                proxy
                    .send_event(message)
                    .expect("failed to send message to event loop");

                buf.clear();
            }
        });

        loop {
            let event = event_recv.recv().await.expect("sender is closed");
            let mut state = state.lock().await;
            match event {
                Event::UserEvent(Message::DeleteWindow(id)) => {
                    drop(state.windows.remove(&id));
                }
                Event::WindowEvent { window_id, event } => {
                    let (id, window) = state
                        .windows
                        .iter_mut()
                        .find(|(_, window)| window.window.id() == window_id)
                        .expect("window event received for nonexistent window");
                    let message = match event {
                        WindowEvent::CloseRequested => Some(ReturnMessage::CloseRequested(*id)),
                        _ => None,
                    };
                    if let Some(message) = message {
                        writer
                            .write(
                                ron::to_string(&message)
                                    .expect("failed to serialize return message")
                                    .as_bytes(),
                            )
                            .await
                            .expect("failed to write to socket");
                    }
                }
                Event::RedrawRequested(window_id) => {
                    let (_, window) = state
                        .windows
                        .iter_mut()
                        .find(|(_, window)| window.window.id() == window_id)
                        .expect("window event received for nonexistent window");
                    window.pixels.render().expect("failed to render window");
                }
                _ => {}
            }
        }
    });

    event_loop.run(move |event, target, c| {
        c.set_wait();

        if let Event::UserEvent(Message::CreateWindow(ref conf, id)) = event {
            let mut state = state.blocking_lock();
            let mut builder = winit::window::WindowBuilder::new()
                .with_visible(conf.visible)
                .with_inner_size(winit::dpi::PhysicalSize::new(conf.size.0, conf.size.1))
                .with_transparent(true)
                .with_title(&conf.title);
            if let Some((x, y)) = conf.pos {
                builder = builder.with_position(winit::dpi::LogicalPosition::new(x, y));
            }
            let window = builder.build(target).expect("failed to create window");
            let surface = pixels::SurfaceTexture::new(conf.size.0, conf.size.1, &window);
            let pixels = pixels::Pixels::new(conf.size.0, conf.size.1, surface)
                .expect("failed to create pixels");
            state.windows.insert(id, Window { window, pixels });
        }

        if let Some(e) = event.to_static() {
            event_send.blocking_send(e).expect("failed to send event");
        }
    })
}
