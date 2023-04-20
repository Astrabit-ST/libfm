#![warn(rust_2018_idioms, clippy::all)]

mod screen;
mod sprite;
mod viewport;

#[macro_export]
macro_rules! send {
    (drop $socket:expr, $msg:expr) => {
        use crate::convert_rust_error;
        use std::io::Write;
        let message = $msg;
        // eprintln!("sending message {:?}", message);

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
        // eprintln!("sending message {:?}", message);

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
