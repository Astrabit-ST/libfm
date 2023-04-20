use screen::Message;

use indexmap::IndexMap;
use std::sync::Arc;

use tokio::sync::{mpsc::unbounded_channel, Mutex};
use winit::event::Event;

mod event_loop;
mod socket_loop;

pub struct State {
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
    let (event_send, event_recv) = unbounded_channel();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("failed to build runtime");
    runtime.spawn(async move {
        let state = async_state;

        tokio::task::spawn(socket_loop::run(proxy));
        event_loop::run(state, event_recv).await;
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
