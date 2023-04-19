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

use magnus::{function, Object};
use std::io::Write;

use crate::{convert_rust_error, screen::Screen};

#[magnus::wrap(class = "LibFM::Viewport", free_immediately, size)]
struct Viewport {
    id: usize,
}

impl Viewport {
    fn new(args: &[magnus::Value]) -> Result<Self, magnus::Error> {
        let args = magnus::scan_args::scan_args::<_, (), (), (), _, ()>(args)?;
        let (screen,): (&Screen,) = args.required;

        let args = magnus::scan_args::get_kwargs::<_, (), _, ()>(
            args.keywords,
            &[],
            &["position", "z", "title", "visible", "size"],
        )?;
        let (pos, z, title, visible, size): (
            Option<_>,
            Option<_>,
            Option<_>,
            Option<_>,
            Option<_>,
        ) = args.optional;

        let title = title.unwrap_or_else(|| "screen exe".to_string());
        let visible = visible.unwrap_or_default();
        let size = size.unwrap_or((640, 480));

        let config = screen::WindowConfig {
            title,
            pos,
            visible,
            size,
            z,
        };
        let id = rand::random();
        let message = screen::Message::CreateWindow(config, id);

        let message = ron::to_string(&message).map_err(convert_rust_error)?;
        let mut socket = screen.socket();
        socket
            .write(message.as_bytes())
            .expect("failed to send config");

        Ok(Viewport { id })
    }
}

pub fn bind(module: &mut impl magnus::Module) -> Result<(), magnus::Error> {
    let class = module.define_class("Viewport", Default::default())?;
    class.define_singleton_method("new", function!(Viewport::new, -1))?;

    Ok(())
}
