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

use screen::Message;
use winit::event_loop::EventLoopProxy;

pub async fn run(
    proxy: EventLoopProxy<Message>,
    mut reader: impl tokio::io::AsyncBufReadExt + Unpin,
) -> ! {
    let mut buf = String::with_capacity(4096);
    loop {
        if let Err(e) = reader.read_line(&mut buf).await {
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
}