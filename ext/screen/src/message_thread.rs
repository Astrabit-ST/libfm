// Copyright (C) 2022 Lily Lyons
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

use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

use crate::CustomEvent;

#[derive(Serialize, Deserialize)]
enum Message {
    Picture(String),
    Position(i32, i32),
    RetrievePosition,
    Title(String),
    Decoration(bool),
    Icon(Option<String>),
    Visible(bool),
}

pub fn message_thread(proxy: EventLoopProxy<CustomEvent>) {
    loop {
        for line in std::io::stdin().lines() {
            let message: Message = ron::from_str(&line.unwrap()).unwrap();

            match message {
                Message::Picture(image) => {
                    let image = image::load_from_memory(&std::fs::read(image).unwrap()).unwrap();

                    proxy.send_event(CustomEvent::ChangeImage(image)).unwrap();
                }
                Message::Position(x, y) => {
                    proxy
                        .send_event(CustomEvent::Move(winit::dpi::PhysicalPosition::new(x, y)))
                        .unwrap();
                }
                Message::RetrievePosition => {
                    proxy.send_event(CustomEvent::GetPosition).unwrap();
                }
                Message::Title(title) => {
                    proxy.send_event(CustomEvent::Title(title)).unwrap();
                }
                Message::Decoration(decoration) => {
                    proxy
                        .send_event(CustomEvent::Decoration(decoration))
                        .unwrap();
                }
                Message::Icon(icon) => {
                    let icon = icon.map(|s| {
                        let icon = image::load_from_memory(&std::fs::read(s).unwrap()).unwrap();

                        let width = icon.width();
                        let height = icon.height();

                        winit::window::Icon::from_rgba(icon.into_bytes(), width, height).unwrap()
                    });

                    proxy.send_event(CustomEvent::Icon(icon)).unwrap();
                }
                Message::Visible(visible) => {
                    proxy.send_event(CustomEvent::Visible(visible)).unwrap();
                }
            }
        }
    }
}
