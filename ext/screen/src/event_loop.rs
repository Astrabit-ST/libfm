// Copyright (C) 2023 Lily Lyons
//
// This file is part of libfm.
//
// libfm is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// libfm is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with libfm.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use crate::{Message, Sprite, State};
use async_bincode::futures::AsyncBincodeWriter;
use futures::prelude::*;
use screen::ReturnMessage;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use winit::event::{Event, WindowEvent};

pub async fn run(
    state: Arc<Mutex<State>>,
    mut event_recv: UnboundedReceiver<Event<'static, Message>>,
    writer: impl AsyncWrite + Unpin,
) -> ! {
    let mut writer = AsyncBincodeWriter::from(writer).for_async();
    loop {
        // Process multiple events at a time in case they have been sent in rapid fire
        let mut events = vec![event_recv.recv().await.expect("sender is closed")];
        while let Ok(event) = event_recv.try_recv() {
            events.push(event);
        }

        let mut state = state.lock().await;
        let State {
            windows,
            wgpu_state,
        } = &mut *state;
        for event in events {
            match event {
                Event::UserEvent(Message::ResizeWindow(width, height, window_id)) => {
                    let window = windows
                        .get_mut(&window_id)
                        .expect("window event recieved for nonexistent window");
                    window
                        .window
                        .set_inner_size(winit::dpi::PhysicalSize::new(width, height));
                    wgpu_state.resize_surface(
                        &mut window.surface,
                        winit::dpi::PhysicalSize::new(width, height),
                    );

                    window.sprites_dirty = true;
                }
                Event::UserEvent(Message::RepositionWindow(x, y, window_id)) => {
                    let window = windows
                        .get_mut(&window_id)
                        .expect("window event recieved for nonexistent window");
                    window
                        .window
                        .set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
                }
                Event::UserEvent(Message::DeleteWindow(id)) => {
                    drop(windows.remove(&id));
                }
                Event::UserEvent(Message::CreateSprite(sprite_id, window_id)) => {
                    let window = windows
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
                    let window = windows
                        .get_mut(&window_id)
                        .expect("window event recieved for nonexistent window");
                    window.sprites_dirty = true;

                    drop(window.sprites.remove(&sprite_id));
                }
                Event::UserEvent(Message::SetSprite(sprite_id, window_id, path)) => {
                    let window = windows
                        .get_mut(&window_id)
                        .expect("window event recieved for nonexistent window");
                    window.sprites_dirty = true;

                    let sprite = window
                        .sprites
                        .get_mut(&sprite_id)
                        .expect("sprite event recieved for nonexistent sprite");
                    sprite.image = Some(wgpu_state.create_texture(path));
                }
                Event::UserEvent(Message::RepositionSprite(sprite_id, window_id, x, y, z)) => {
                    let window = windows
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

                Event::WindowEvent { window_id, event } => {
                    let (id, window) = windows
                        .iter_mut()
                        .find(|(_, window)| window.window.id() == window_id)
                        .expect("window event received for nonexistent window");
                    let message = match event {
                        WindowEvent::CloseRequested => Some(ReturnMessage::CloseRequested(*id)),
                        _ => None,
                    };
                    if let Some(message) = message {
                        writer
                            .send(message)
                            .await
                            .expect("failed to send response message");
                    }
                }

                Event::RedrawRequested(window_id) => {
                    let (_, window) = windows
                        .iter_mut()
                        .find(|(_, window)| window.window.id() == window_id)
                        .expect("window event received for nonexistent window");
                    let output = window.surface.get_current_texture();

                    let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = wgpu_state.create_command_encoder();

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.05,
                                    g: 0.0,
                                    b: 0.1,
                                    a: 0.7,
                                }),
                                store: true,
                            },
                        })],
                        ..Default::default()
                    });

                    for sprite in window.sprites.values() {
                        let Some(ref texture) = sprite.image else { continue; };
                        texture.bind(&mut render_pass);
                        wgpu_state.sprite_shader.bind(&mut render_pass);

                        render_pass.draw(0..3, 0..1);
                    }

                    drop(render_pass);

                    wgpu_state.submit_encoder(encoder);
                    output.present();
                }
                _ => {}
            }
        }

        for window in state.windows.values_mut() {
            if window.sprites_dirty {
                window
                    .sprites
                    .sort_unstable_by(|_, s, _, s2| s.z.cmp(&s2.z));
                window.window.request_redraw();
                window.sprites_dirty = false;
            }
        }
    }
}
