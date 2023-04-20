use screen::Message;

use indexmap::IndexMap;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc::unbounded_channel, Mutex};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use winit::event::Event;

struct State {
    windows: IndexMap<usize, Window>,
}

struct Window {
    window: winit::window::Window,
    pixels: pixels::Pixels,
    sprites: IndexMap<usize, Sprite>,
    sprites_dirty: bool,
}

struct Sprite {
    x: i32,
    y: i32,
    z: i32,
    image: Option<image::RgbaImage>,
}

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let state = Arc::new(Mutex::new(State {
        windows: IndexMap::new(),
    }));
    let async_state = state.clone();
    let (event_send, mut event_recv) = unbounded_channel();

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
        let mut reader = BufReader::new(socket.compat());

        tokio::task::spawn(async move {
            let mut buf = String::with_capacity(4096);
            loop {
                // eprintln!("starting to read socket");
                if let Err(e) = reader.read_line(&mut buf).await {
                    eprintln!("error reading socket buffer: {e:?}")
                }
                let Ok(message) = ron::from_str::<Message>(&buf) else {
                    eprintln!("error reading message");

                    continue;
                };
                // eprintln!("got message {message:?}");
                proxy
                    .send_event(message)
                    .expect("failed to send message to event loop");

                buf.clear();
            }
        });

        loop {
            // Process multiple events at a time in case they have been sent in rapid fire
            let mut events = vec![event_recv.recv().await.expect("sender is closed")];
            while let Ok(event) = event_recv.try_recv() {
                events.push(event);
            }

            let mut state = state.lock().await;
            for event in events {
                match event {
                    Event::UserEvent(Message::ResizeWindow(width, height, window_id)) => {
                        let window = state
                            .windows
                            .get_mut(&window_id)
                            .expect("window event recieved for nonexistent window");
                        window
                            .window
                            .set_inner_size(winit::dpi::PhysicalSize::new(width, height));
                        window
                            .pixels
                            .resize_buffer(width, height)
                            .expect("failed to resize pixel buffer");
                        window
                            .pixels
                            .resize_surface(width, height)
                            .expect("failed to resize window surface");
                        window.sprites_dirty = true;
                    }
                    Event::UserEvent(Message::RepositionWindow(x, y, window_id)) => {
                        let window = state
                            .windows
                            .get_mut(&window_id)
                            .expect("window event recieved for nonexistent window");
                        window
                            .window
                            .set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
                    }
                    Event::UserEvent(Message::DeleteWindow(id)) => {
                        drop(state.windows.remove(&id));
                    }
                    Event::UserEvent(Message::CreateSprite(sprite_id, window_id)) => {
                        let window = state
                            .windows
                            .get_mut(&window_id)
                            .expect("window event recieved for nonexistent window");
                        window.sprites.insert(
                            sprite_id,
                            Sprite {
                                x: 0,
                                y: 0,
                                z: 0,
                                image: None,
                            },
                        );
                    }
                    Event::UserEvent(Message::RemoveSprite(sprite_id, window_id)) => {
                        let window = state
                            .windows
                            .get_mut(&window_id)
                            .expect("window event recieved for nonexistent window");
                        window.sprites_dirty = true;
                        drop(window.sprites.remove(&sprite_id));
                    }
                    Event::UserEvent(Message::SetSprite(sprite_id, window_id, path)) => {
                        let window = state
                            .windows
                            .get_mut(&window_id)
                            .expect("window event recieved for nonexistent window");
                        window.sprites_dirty = true;
                        let sprite = window
                            .sprites
                            .get_mut(&sprite_id)
                            .expect("sprite event recieved for nonexistent sprite");
                        sprite.image = Some(
                            image::open(path)
                                .expect("failed to load image")
                                .into_rgba8(),
                        );
                    }
                    Event::UserEvent(Message::RepositionSprite(sprite_id, window_id, x, y, z)) => {
                        let window = state
                            .windows
                            .get_mut(&window_id)
                            .expect("window event recieved for nonexistent window");
                        window.sprites_dirty = true;
                        let sprite = window
                            .sprites
                            .get_mut(&sprite_id)
                            .expect("sprite event recieved for nonexistent sprite");
                        sprite.x = x;
                        sprite.y = y;
                        sprite.z = z;
                    }
                    /*
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
                    */
                    Event::RedrawRequested(window_id) => {
                        let (_, window) = state
                            .windows
                            .iter_mut()
                            .find(|(_, window)| window.window.id() == window_id)
                            .expect("window event received for nonexistent window");

                        if window.sprites_dirty {
                            window
                                .sprites
                                .sort_unstable_by(|_, s, _, s2| s.z.cmp(&s2.z));
                            let size = window.pixels.texture().size();
                            let buffer = window.pixels.frame_mut();

                            buffer.fill(0);
                            for sprite in window.sprites.values() {
                                if let Some(image) = &sprite.image {
                                    for (x, y, pixel) in image.enumerate_pixels() {
                                        let x = sprite.x + x as i32;
                                        let y = sprite.y + y as i32;
                                        if x.is_negative()
                                            || y.is_negative()
                                            || x as u32 >= size.width
                                            || y as u32 >= size.height
                                        {
                                            continue;
                                        }

                                        let start_index =
                                            ((y * size.width as i32 * 4) + (x * 4)) as usize;

                                        buffer[start_index] = pixel[0];
                                        buffer[start_index + 1] = pixel[1];
                                        buffer[start_index + 2] = pixel[2];
                                        buffer[start_index + 3] = pixel[3];
                                    }
                                }
                            }
                            window.sprites_dirty = false;
                        }
                        window.pixels.render().expect("failed to render window");
                    }
                    _ => {}
                }
            }

            for window in state.windows.values() {
                if window.sprites_dirty {
                    window.window.request_redraw();
                }
            }
        }
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
            let surface = pixels::SurfaceTexture::new(conf.size.0, conf.size.1, &window);
            let pixels = pixels::PixelsBuilder::new(conf.size.0, conf.size.1, surface)
                .clear_color(pixels::wgpu::Color::TRANSPARENT)
                .build()
                .expect("failed to create pixels");
            state.windows.insert(
                id,
                Window {
                    window,
                    pixels,
                    sprites: IndexMap::new(),
                    sprites_dirty: false,
                },
            );

            c.set_wait_timeout(std::time::Duration::from_millis(16));
        }

        if let Some(e) = event.to_static() {
            event_send.send(e).expect("failed to send event");
        }
    })
}
