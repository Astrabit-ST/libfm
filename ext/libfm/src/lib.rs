#![warn(rust_2018_idioms, clippy::all)]

mod viewport;
mod screen;

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

    Ok(())
}
