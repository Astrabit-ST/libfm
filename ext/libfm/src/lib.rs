#![warn(rust_2018_idioms, clippy::all)]

use magnus::function;
use std::io::{prelude::*, BufReader};

mod screen;

fn exec_no_cmd(cmd: String, args: Vec<String>) -> String {
    let stdout = subprocess::Exec::cmd(cmd)
        .args(args.as_slice())
        .stream_stdout()
        .unwrap();
    let mut br = BufReader::new(stdout);
    let mut buf = String::new();
    br.read_to_string(&mut buf).unwrap();

    buf
}

#[magnus::init]
fn init() -> Result<(), magnus::Error> {
    let module = magnus::define_module("LibFM")?;
    module.define_module_function("exec_no_cmd", function!(exec_no_cmd, 2))?;

    screen::bind(module)
}
