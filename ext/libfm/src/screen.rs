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
#![allow(unused_variables)]

use std::ffi::CString;

use crossbeam_channel::{Receiver, RecvError, Sender};
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::*,
};
use glutin_winit::DisplayBuilder;
use magnus::{function, method, Module, Object};

use once_cell::sync::OnceCell;
use raw_window_handle::HasRawWindowHandle;
#[cfg(target_os = "unix")]
use winit::platform::unix::EventLoopBuilderExtUnix;
#[cfg(target_os = "windows")]
use winit::platform::windows::{EventLoopBuilderExtWindows, WindowBuilderExtWindows};
use winit::{
    event_loop::EventLoopBuilder,
    window::{Window, WindowBuilder},
};

static CREATE_WINDOW: OnceCell<Sender<WindowBuilder>> = OnceCell::new();
static GET_WINDOW: OnceCell<Receiver<(Window, Config)>> = OnceCell::new();

#[magnus::wrap(class = "LibFM::Screen", free_immediately, size)]
struct Screen {
    sender: Sender<Message>,
    result: Receiver<Return>,
    handle: std::thread::JoinHandle<()>,
}

impl Screen {
    fn new() -> Self {
        let (sender, reciever) = crossbeam_channel::unbounded();
        let (sender_r, reciever_r) = crossbeam_channel::unbounded();

        let handle = std::thread::spawn(move || {
            unsafe {
                Self::screen_thread(reciever, sender_r);
            };
        });

        Self {
            sender,
            result: reciever_r,
            handle,
        }
    }

    fn finished(&self) -> bool {
        self.handle.is_finished()
    }

    fn set(&self, image: String) {
        self.sender.send(Message::Picture(image)).unwrap();
    }

    fn move_(&self, x: i32, y: i32) {
        self.sender.send(Message::Position(x, y)).unwrap();
    }

    fn pos(&self) -> (i32, i32) {
        self.sender.send(Message::RetrievePosition).unwrap();
        self.result.recv().unwrap().into_position().unwrap()
    }

    fn title(&self, title: String) {
        self.sender.send(Message::Title(title)).unwrap();
    }

    fn decoration(&self, decoration: bool) {
        self.sender.send(Message::Decoration(decoration)).unwrap()
    }

    fn icon(&self, icon: Option<String>) {
        self.sender.send(Message::Icon(icon)).unwrap();
    }

    fn close(&self) {
        self.sender.send(Message::Close).unwrap()
    }

    fn visible(&self, visible: bool) {
        self.sender.send(Message::Visible(visible)).unwrap()
    }

    unsafe fn screen_thread(reciever: Receiver<Message>, sender_r: Sender<Return>) -> ! {
        let window_builder = WindowBuilder::new()
            .with_transparent(true)
            .with_always_on_top(true)
            .with_decorations(false)
            .with_visible(false)
            .with_resizable(false);

        #[cfg(target_os = "windows")]
        let window_builder = window_builder.with_skip_taskbar(true);

        let (window, gl_config) = {
            CREATE_WINDOW.get().unwrap().send(window_builder).unwrap();

            GET_WINDOW.get().unwrap().recv().unwrap()
        };

        let gl_display = gl_config.display();

        gl_display
            .create_context(
                &gl_config,
                &ContextAttributesBuilder::new().build(Some(window.raw_window_handle())),
            )
            .unwrap();

        let gl = glow::Context::from_loader_function(|sym| {
            let sym = CString::new(sym).unwrap();
            gl_display.get_proc_address(&sym)
        });

        loop {
            let message = reciever.recv();
            match message {
                Ok(message) => match message {
                    Message::Picture(image) => {
                        println!("Loading {image}")
                    }
                    Message::Position(x, y) => {
                        window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y))
                    }
                    Message::RetrievePosition => {
                        let position = window.outer_position().unwrap();
                        sender_r
                            .send(Return::Position(position.x, position.y))
                            .unwrap()
                    }
                    Message::Title(title) => window.set_title(&title),
                    Message::Decoration(decoration) => window.set_decorations(decoration),
                    Message::Icon(icon) => {
                        let icon = icon.map(|s| {
                            let icon = image::load_from_memory(&std::fs::read(s).unwrap()).unwrap();

                            let width = icon.width();
                            let height = icon.height();

                            winit::window::Icon::from_rgba(icon.into_bytes(), width, height)
                                .unwrap()
                        });

                        window.set_window_icon(icon)
                    }
                    Message::Close => panic!("You closed the window LMAO"),
                    Message::Visible(visible) => window.set_visible(visible),
                },
                Err(RecvError) => {
                    panic!("Channel disconnected, exiting...");
                }
            }
        }
    }
}

enum Message {
    Picture(String),
    Position(i32, i32),
    RetrievePosition,
    Title(String),
    Decoration(bool),
    Icon(Option<String>),
    Close,
    Visible(bool),
}

#[derive(Debug, enum_as_inner::EnumAsInner)]
enum Return {
    Position(i32, i32),
}

pub fn bind(module: impl magnus::Module) -> Result<(), magnus::Error> {
    let class = module.define_class("Screen", Default::default())?;
    class.define_singleton_method("new", function!(Screen::new, 0))?;
    class.define_method("finished", method!(Screen::finished, 0))?;
    class.define_method("set", method!(Screen::set, 1))?;
    class.define_method("move", method!(Screen::move_, 2))?;
    class.define_method("pos", method!(Screen::pos, 0))?;
    class.define_method("title", method!(Screen::title, 1))?;
    class.define_method("decoration", method!(Screen::decoration, 1))?;
    class.define_method("icon", method!(Screen::icon, 1))?;
    class.define_method("close", method!(Screen::close, 0))?;
    class.define_method("visible", method!(Screen::visible, 1))?;

    let (window_sender, window_reciever) = crossbeam_channel::unbounded();
    let (builder_sender, builder_reciever) = crossbeam_channel::unbounded();
    GET_WINDOW.set(window_reciever).unwrap();
    CREATE_WINDOW.set(builder_sender).unwrap();

    std::thread::spawn(move || {
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
        event_loop.run(move |_event, target, control_flow| {
            let iter = builder_reciever.try_iter();

            for window_builder in iter {
                let template = ConfigTemplateBuilder::new().with_alpha_size(8);

                let display_builder =
                    DisplayBuilder::new().with_window_builder(Some(window_builder));

                let (window, gl_config) = display_builder
                    .build(target, template, |configs| {
                        configs
                            .reduce(|accum, config| {
                                let transparency_check =
                                    config.supports_transparency().unwrap_or(false)
                                        & !accum.supports_transparency().unwrap_or(false);

                                if transparency_check || config.num_samples() > accum.num_samples()
                                {
                                    config
                                } else {
                                    accum
                                }
                            })
                            .unwrap()
                    })
                    .unwrap();

                window_sender.send((window.unwrap(), gl_config)).unwrap();
            }
        })
    });

    Ok(())
}
