#![warn(rust_2018_idioms, clippy::all)]

mod screen;
mod sprite;
mod viewport;

#[macro_export]
macro_rules! send {
    ($screen:expr, $msg:expr) => {
        use crate::convert_rust_error;
        use futures::prelude::*;

        let message = $msg;
        let mut m_lock = $screen.lock();
        let lock = &mut *m_lock;

        lock.runtime
            .block_on(async { lock.writer.send(message).await.map_err(convert_rust_error) })?;

        drop(lock);
        drop(m_lock);
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
