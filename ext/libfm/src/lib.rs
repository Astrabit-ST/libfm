#![warn(rust_2018_idioms, clippy::all)]

mod screen;

#[magnus::init]
fn init() -> Result<(), magnus::Error> {
    let module = magnus::define_module("LibFM")?;

    screen::bind(module)
}
