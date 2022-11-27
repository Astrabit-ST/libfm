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

use std::sync::Arc;

use crossbeam_channel::Sender;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use winit::window::Window;

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

pub fn message_thread(window: Arc<Window>, image_sender: Sender<DynamicImage>) {
    loop {
        for line in std::io::stdin().lines() {
            let message: Message = ron::from_str(&line.unwrap()).unwrap();

            match message {
                Message::Picture(image) => {
                    eprintln!("Loading {image}");
                    let image = image::load_from_memory(&std::fs::read(image).unwrap()).unwrap();
                    image_sender.send(image).unwrap();
                }
                Message::Position(x, y) => {
                    window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y))
                }
                Message::RetrievePosition => {
                    let position = window.outer_position().unwrap();

                    println!("{}", ron::to_string(&(position.x, position.y)).unwrap());
                }
                Message::Title(title) => window.set_title(&title),
                Message::Decoration(decoration) => window.set_decorations(decoration),
                Message::Icon(icon) => {
                    let icon = icon.map(|s| {
                        let icon = image::load_from_memory(&std::fs::read(s).unwrap()).unwrap();

                        let width = icon.width();
                        let height = icon.height();

                        winit::window::Icon::from_rgba(icon.into_bytes(), width, height).unwrap()
                    });

                    window.set_window_icon(icon)
                }
                Message::Visible(visible) => {
                    window.set_visible(visible);
                }
            }
        }
    }
}
