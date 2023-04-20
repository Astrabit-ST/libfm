#![warn(rust_2018_idioms, clippy::all)]

use std::sync::Arc;

use interprocess::local_socket;

mod screen;
mod sprite;
mod viewport;

#[macro_export]
macro_rules! send {
    (drop $socket:expr, $msg:expr) => {
        use crate::convert_rust_error;
        use std::io::Write;
        let message = $msg;

        let mut socket = $socket;
        socket
            .write(
                ron::to_string(&message)
                    .map_err(convert_rust_error)?
                    .as_bytes(),
            )
            .and_then(|_| socket.write(&[b'\n']))
            .map_err(convert_rust_error)?;
        drop(socket);
    };
    ($socket:expr, $msg:expr) => {
        use crate::convert_rust_error;
        use std::io::Write;
        let message = $msg;

        $socket
            .write(
                ron::to_string(&message)
                    .map_err(convert_rust_error)?
                    .as_bytes(),
            )
            .and_then(|_| $socket.write(&[b'\n']))
            .map_err(convert_rust_error)?;
    };
}

pub fn convert_rust_error(error: impl ToString) -> magnus::Error {
    magnus::Error::new(magnus::exception::runtime_error(), error.to_string())
}

#[magnus::init]
fn init() -> Result<(), magnus::Error> {
    unsafe {
        rb_sys::rb_ext_ractor_safe(true);
    }

    let mut module = magnus::define_module("LibFM")?;
    viewport::bind(&mut module)?;
    screen::bind(&mut module)?;
    sprite::bind(&mut module)?;

    Ok(())
}

struct SocketReader(Arc<local_socket::LocalSocketStream>);

//? SAFETY: I've read of the code of local_socket::LocalSocketStream.
//? read + write do not mutate each other, so this is okay.
impl std::io::Read for SocketReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            // Get a mutable reference out of the arc. This is fine, because again, I've checked the code.
            let inner_mut_ref = &mut *Arc::as_ptr(&self.0).cast_mut();
            inner_mut_ref.read(buf)
        }
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        unsafe {
            // Get a mutable reference out of the arc. This is fine, because again, I've checked the code.
            let inner_mut_ref = &mut *Arc::as_ptr(&self.0).cast_mut();
            inner_mut_ref.read_vectored(bufs)
        }
    }
}

struct SocketWriter(Arc<local_socket::LocalSocketStream>);

//? SAFETY: I've read of the code of local_socket::LocalSocketStream.
//? read + write do not mutate each other, so this is okay.
impl std::io::Write for SocketWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            // Get a mutable reference out of the arc. This is fine, because again, I've checked the code.
            let inner_mut_ref = &mut *Arc::as_ptr(&self.0).cast_mut();
            inner_mut_ref.write(buf)
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        unsafe {
            // Get a mutable reference out of the arc. This is fine, because again, I've checked the code.
            let inner_mut_ref = &mut *Arc::as_ptr(&self.0).cast_mut();
            inner_mut_ref.write_vectored(bufs)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unsafe {
            // Get a mutable reference out of the arc. This is fine, because again, I've checked the code.
            let inner_mut_ref = &mut *Arc::as_ptr(&self.0).cast_mut();
            inner_mut_ref.flush()
        }
    }
}

fn into_split(socket: local_socket::LocalSocketStream) -> (SocketReader, SocketWriter) {
    let inner = Arc::new(socket);
    (SocketReader(inner.clone()), SocketWriter(inner))
}
