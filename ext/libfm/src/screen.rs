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
#![allow(unused_variables, dead_code)]

use parking_lot::Mutex;

use magnus::{function, method, Module, Object};
use serde::{Deserialize, Serialize};
use std::io::{prelude::*, BufReader};
use subprocess::{Exec, Popen, Redirection};

#[magnus::wrap(class = "LibFM::Screen", free_immediately, size)]
struct Screen {
    popen: Mutex<Popen>,
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = self.popen.get_mut().kill();
    }
}

impl Screen {
    fn new() -> Self {
        let popen = Exec::cmd("lib/screen")
            .stdin(Redirection::Pipe)
            .stdout(Redirection::Pipe)
            .popen()
            .unwrap()
            .into();

        Self { popen }
    }

    fn active(&self) -> bool {
        let mut popen = self.popen.lock();
        popen.poll().is_none()
    }

    fn stop(&self) {
        let mut popen = self.popen.lock();
        let _ = popen.kill();
    }

    fn write<T>(&self, data: T)
    where
        T: serde::Serialize,
    {
        let mut popen = self.popen.lock();
        let str = ron::to_string(&data).unwrap();

        writeln!(popen.stdin.as_mut().unwrap(), "{str}").unwrap();
    }

    fn visible(&self, visible: bool) {
        self.write(Message::Visible(visible));
    }

    fn set(&self, image: String) {
        self.write(Message::Picture(image));
    }

    fn move_(&self, x: i32, y: i32) {
        self.write(Message::Position(x, y));
    }

    fn pos(&self) -> (i32, i32) {
        self.write(Message::RetrievePosition);

        let mut buf = String::new();
        let mut popen = self.popen.lock();
        BufReader::new(popen.stdout.as_mut().unwrap())
            .read_line(&mut buf)
            .unwrap();
        ron::from_str(&buf).unwrap()
    }

    fn title(&self, title: String) {
        self.write(Message::Title(title));
    }

    fn decoration(&self, decoration: bool) {
        self.write(Message::Decoration(decoration));
    }

    fn icon(&self, icon: Option<String>) {
        self.write(Message::Icon(icon));
    }
}

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

pub fn bind(module: impl magnus::Module) -> Result<(), magnus::Error> {
    let class = module.define_class("Screen", Default::default())?;
    class.define_singleton_method("new", function!(Screen::new, 0))?;
    class.define_method("active", method!(Screen::active, 0))?;
    class.define_method("stop", method!(Screen::stop, 0))?;
    class.define_method("visible", method!(Screen::visible, 1))?;
    class.define_method("set", method!(Screen::set, 1))?;
    class.define_method("move", method!(Screen::move_, 2))?;
    class.define_method("pos", method!(Screen::pos, 0))?;
    class.define_method("title", method!(Screen::title, 1))?;
    class.define_method("decoration", method!(Screen::decoration, 1))?;
    class.define_method("icon", method!(Screen::icon, 1))?;

    Ok(())
}
