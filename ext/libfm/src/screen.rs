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
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};

use crate::convert_rust_error;
use interprocess::local_socket;

use std::sync::Arc;

macro_rules! gaurd_dead {
    ($child:expr) => {
        match $child.try_wait() {
            Ok(Some(c)) => {
                return Err(magnus::Error::new(
                    magnus::exception::runtime_error(),
                    format!("child process is dead with code {:?}", c.code()),
                ))
            }
            Err(e) => {
                return Err(magnus::Error::new(
                    magnus::exception::runtime_error(),
                    e.to_string(),
                ))
            }
            Ok(_) => {}
        };
    };
}

struct Inner {
    child: std::process::Child,
    socket: local_socket::LocalSocketStream,
}

impl Drop for Inner {
    fn drop(&mut self) {
        let _ = self.child.kill();
        self.child.wait().expect("failed to wait on child");
    }
}

#[magnus::wrap(class = "LibFM::Screen", free_immediately, size)]
#[derive(Clone)]
pub struct Screen {
    inner: Arc<Mutex<Inner>>,
}

impl Screen {
    fn new(args: &[magnus::Value]) -> Result<Self, magnus::Error> {
        let args = magnus::scan_args::scan_args::<(), (), (), (), _, ()>(args)?;
        let args = magnus::scan_args::get_kwargs::<_, (), _, ()>(
            args.keywords,
            &[],
            &["screen_path", "socket_addr"],
        )?;
        let (screen_path, socket_addr): (Option<_>, Option<String>) = args.optional;

        let screen_path = screen_path.unwrap_or_else(|| "target/release/screen".to_string());

        let socket_addr = socket_addr.unwrap_or_else(|| "abcdef".to_string());
        let socket_addr = match local_socket::NameTypeSupport::query() {
            local_socket::NameTypeSupport::OnlyPaths => {
                format!("/tmp/libfm-screen-sock-{socket_addr}.sock")
            }
            local_socket::NameTypeSupport::Both | local_socket::NameTypeSupport::OnlyNamespaced => {
                format!("@libfm-screen-sock-{socket_addr}.sock")
            }
        };

        // eprintln!("connecting to socket at {socket_addr}");

        let listener = local_socket::LocalSocketListener::bind(socket_addr.clone())
            .map_err(convert_rust_error)?;

        let mut child = std::process::Command::new(screen_path)
            .arg(socket_addr)
            .spawn()
            .map_err(convert_rust_error)?;

        gaurd_dead!(child);

        let socket = listener.accept().map_err(convert_rust_error)?;

        // eprintln!("connected");

        Ok(Self {
            inner: Arc::new(Mutex::new(Inner { child, socket })),
        })
    }

    fn is_alive(&self) -> bool {
        self.inner
            .lock()
            .child
            .try_wait()
            .is_ok_and(|c| c.is_none())
    }

    pub fn socket(&self) -> MappedMutexGuard<'_, local_socket::LocalSocketStream> {
        MutexGuard::map(self.inner.lock(), |i| &mut i.socket)
    }
}

pub fn bind(module: &mut impl magnus::Module) -> Result<(), magnus::Error> {
    let class = module.define_class("Screen", Default::default())?;
    class.define_singleton_method("new", function!(Screen::new, -1))?;
    class.define_method("alive?", method!(Screen::is_alive, 0))?;

    Ok(())
}
