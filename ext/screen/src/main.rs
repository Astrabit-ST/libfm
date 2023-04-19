use screen::Message;
use std::io::{prelude::*, BufReader};
use winit::event::Event;

struct State {
    windows: Vec<winit::window::Window>,
}

fn main() {
    let socket_addr = std::env::args().nth(1).expect("socket addr not provided");
    let socket = interprocess::local_socket::LocalSocketStream::connect(socket_addr)
        .expect("failed to connect to socket");
    let mut socket = BufReader::new(socket);

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    std::thread::spawn(move || {
        let mut buf = String::with_capacity(4096);
        loop {
            if let Err(e) = socket.read_line(&mut buf) {
                eprintln!("error reading socket buffer: {e:?}")
            }
            let Ok(message) = ron::from_str::<Message>(&buf) else {
                eprintln!("error reading message");

                continue;
            };
            proxy
                .send_event(message)
                .expect("failed to send message to event loop");

            buf.clear();
        }
    });

    let state = State { windows: vec![] };

    event_loop.run(move |event, b, c| {
        c.set_wait();

        match event {
            Event::UserEvent(m) => {}
            _ => {}
        }
    })
}
