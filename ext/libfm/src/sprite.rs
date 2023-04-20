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

use crate::{screen::Screen, send, viewport::Viewport};
use magnus::{function, method, Module, Object};
use parking_lot::Mutex;
use screen::Message;
use std::io::Write;

#[magnus::wrap(class = "LibFM::Sprite", free_immediately, size)]
struct Sprite {
    id: usize,
    viewport_id: usize,
    screen: Screen,
    position: Mutex<(i32, i32)>,
}

impl Drop for Sprite {
    fn drop(&mut self) {
        self.close();
    }
}

impl Sprite {
    pub fn new(viewport: &Viewport) -> Result<Self, magnus::Error> {
        let screen = viewport.screen.clone();

        let socket = screen.socket();
        let id = rand::random();
        send!(drop socket, Message::CreateSprite(id, viewport.id));

        Ok(Self {
            id,
            screen,
            viewport_id: viewport.id,
            position: Mutex::new((0, 0)),
        })
    }

    fn close(&self) {
        let mut socket = self.screen.socket();
        let text = match ron::to_string(&Message::RemoveSprite(self.id, self.viewport_id)) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error serializing {e:?}");
                return;
            }
        };
        if let Err(e) = socket
            .write(text.as_bytes())
            .and_then(|_| socket.write(&[b'\n']))
        {
            eprintln!("error sending message {e:?}")
        }
    }

    fn set(&self, filename: String) -> Result<(), magnus::Error> {
        if !std::path::Path::new(&filename).exists() {
            return Err(magnus::Error::new(
                magnus::exception::io_error(),
                format!("File does not exist {filename}"),
            ));
        }
        send!(
            drop self.screen.socket(),
            Message::SetSprite(self.id, self.viewport_id, filename)
        );

        Ok(())
    }

    fn reposition(&self, x: i32, y: i32) -> Result<(), magnus::Error> {
        *self.position.lock() = (x, y);
        send!(
            drop self.screen.socket(),
            Message::RepositionSprite(self.id, self.viewport_id, x, y)
        );

        Ok(())
    }

    fn get_x(&self) -> i32 {
        self.position.lock().0
    }

    fn set_x(&self, x: i32) -> Result<(), magnus::Error> {
        self.reposition(x, self.get_y())
    }

    fn get_y(&self) -> i32 {
        self.position.lock().1
    }

    fn set_y(&self, y: i32) -> Result<(), magnus::Error> {
        self.reposition(self.get_x(), y)
    }
}

pub fn bind(module: &mut impl magnus::Module) -> Result<(), magnus::Error> {
    let class = module.define_class("Sprite", Default::default())?;
    class.define_singleton_method("new", function!(Sprite::new, 1))?;
    class.define_method("close", method!(Sprite::close, 0))?;
    class.define_method("set", method!(Sprite::set, 1))?;
    class.define_method("move", method!(Sprite::reposition, 2))?;

    class.define_method("x", method!(Sprite::get_x, 0))?;
    class.define_method("x=", method!(Sprite::set_x, 1))?;
    class.define_method("y", method!(Sprite::get_y, 0))?;
    class.define_method("y=", method!(Sprite::set_y, 1))?;

    Ok(())
}
