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

use magnus::{function, method, Module, Object};
use screen::Message;

use crate::{screen::Screen, send};

#[magnus::wrap(class = "LibFM::Viewport", free_immediately, size)]
pub struct Viewport {
    pub id: usize,
    pub screen: Screen,
}

impl Drop for Viewport {
    fn drop(&mut self) {
        self.close();
    }
}

impl Viewport {
    fn new(args: &[magnus::Value]) -> Result<Self, magnus::Error> {
        let args = magnus::scan_args::scan_args::<_, (), (), (), _, ()>(args)?;
        let (screen,): (&Screen,) = args.required;

        let args = magnus::scan_args::get_kwargs::<_, (), _, ()>(
            args.keywords,
            &[],
            &["position", "z", "title", "visible", "size", "decorations"],
        )?;
        let (pos, z, title, visible, size, decorations): (
            Option<_>,
            Option<_>,
            Option<_>,
            Option<_>,
            Option<_>,
            Option<_>,
        ) = args.optional;

        let title = title.unwrap_or_else(|| "screen exe".to_string());
        let visible = visible.unwrap_or_default();
        let decorations = decorations.unwrap_or_default();
        let size = size.unwrap_or((640, 480));

        let config = screen::WindowConfig {
            title,
            pos,
            visible,
            decorations,
            size,
            z,
        };
        let id = rand::random();

        send!(screen, screen::Message::CreateWindow(config, id));

        Ok(Viewport {
            id,
            screen: screen.clone(),
        })
    }

    fn reposition(&self, x: i32, y: i32) -> Result<(), magnus::Error> {
        send!(self.screen, Message::RepositionWindow(x, y, self.id));

        Ok(())
    }

    fn resize(&self, x: u32, y: u32) -> Result<(), magnus::Error> {
        send!(self.screen, Message::ResizeWindow(x, y, self.id));

        Ok(())
    }

    fn close(&self) {
        use futures::prelude::*;

        let lock = &mut *self.screen.lock();
        lock.runtime.block_on(async {
            if let Err(e) = lock.writer.send(Message::DeleteWindow(self.id)).await {
                eprintln!("error sending message {e:?}")
            }
        });
    }
}

pub fn bind(module: &mut impl magnus::Module) -> Result<(), magnus::Error> {
    let class = module.define_class("Viewport", Default::default())?;
    class.define_singleton_method("new", function!(Viewport::new, -1))?;
    class.define_method("move", method!(Viewport::reposition, 2))?;
    class.define_method("close", method!(Viewport::close, 0))?;
    class.define_method("resize", method!(Viewport::resize, 2))?;

    Ok(())
}
