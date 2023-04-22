use screen::Message;

use indexmap::IndexMap;
use std::sync::Arc;

use tokio::sync::{mpsc::unbounded_channel, Mutex};
use winit::event::Event;

mod event_loop;
mod socket_loop;
mod wgpu_state;

pub struct State {
    windows: IndexMap<usize, Window>,
    wgpu_state: wgpu_state::State,
}

struct Window {
    window: winit::window::Window,
    surface: wgpu_state::Surface,
    sprites: IndexMap<usize, Sprite>,
    sprites_dirty: bool,
}

struct Sprite {
    x: i32,
    y: i32,
    z: i32,
    image: Option<wgpu_state::Texture>,
}

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("failed to build runtime");

    let state = Arc::new(Mutex::new(State {
        windows: IndexMap::new(),
        wgpu_state: runtime.block_on(wgpu_state::State::new()),
    }));
    let async_state = state.clone();
    let (event_send, event_recv) = unbounded_channel();

    runtime.spawn(async move {
        let state = async_state;
        let socket_addr = std::env::args().nth(1).expect("socket addr not provided");
        let socket = interprocess::local_socket::tokio::LocalSocketStream::connect(socket_addr)
            .await
            .expect("failed to connect to socket");

        let (reader, writer) = socket.into_split();

        tokio::task::spawn(socket_loop::run(proxy, reader));
        event_loop::run(state, event_recv, writer).await;
    });

    event_loop.run(move |event, target, c| {
        c.set_wait_timeout(std::time::Duration::from_millis(16));

        let mut state = state.blocking_lock();
        if let Event::UserEvent(Message::CreateWindow(ref conf, id)) = event {
            let mut builder = winit::window::WindowBuilder::new()
                .with_visible(conf.visible)
                .with_inner_size(winit::dpi::PhysicalSize::new(conf.size.0, conf.size.1))
                .with_transparent(true)
                .with_decorations(conf.decorations)
                .with_resizable(false)
                .with_title(&conf.title);
            if let Some((x, y)) = conf.pos {
                builder = builder.with_position(winit::dpi::LogicalPosition::new(x, y));
            }
            let window = builder.build(target).expect("failed to create window");
            let surface = state.wgpu_state.create_surface(&window);

            state.windows.insert(
                id,
                Window {
                    window,
                    sprites: IndexMap::new(),
                    sprites_dirty: false,
                    surface,
                },
            );
        }

        if let Some(e) = event.to_static() {
            event_send.send(e).expect("failed to send event");
        }
    })
}
