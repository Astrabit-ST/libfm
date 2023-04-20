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
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use winit::event::Event;

pub async fn run(
    state: Arc<Mutex<State>>,
    mut event_recv: UnboundedReceiver<Event<'static, Message>>,
) -> ! {
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
}
