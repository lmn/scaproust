// Copyright 2016 Benoît Labaere (benoit.labaere@gmail.com)
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

use std::rc::Rc;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::io;

use mio;

use byteorder::*;

use protocol::Protocol;
use protocol::policy::*;
use transport::pipe::Pipe;
use global::*;
use event_loop_msg::{ SocketNotify };
use EventLoop;
use Message;

pub struct Bus {
    id: SocketId,
    body: Body,
    state: Option<State>
}

struct Body {
    notify_sender: Rc<Sender<SocketNotify>>,
    pipes: HashMap<mio::Token, Pipe>,
    fq: PrioList
}

enum State {
    Idle,
    Receiving(mio::Token, Timeout),
    RecvOnHold(Timeout)
}

impl Bus {
    pub fn new(socket_id: SocketId, notify_tx: Rc<Sender<SocketNotify>>) -> Bus {
        let body = Body {
            notify_sender: notify_tx,
            pipes: HashMap::new(),
            fq: PrioList::new()
        };

        Bus {
            id: socket_id,
            body: body,
            state: Some(State::Idle)
        }
    }

    fn apply<F>(&mut self, transition: F) where F : FnOnce(State, &mut Body) -> State {
        if let Some(old_state) = self.state.take() {
            let old_name = old_state.name();
            let new_state = transition(old_state, &mut self.body);
            let new_name = new_state.name();

            self.state = Some(new_state);

            debug!("[{:?}] switch from '{}' to '{}'.", self.id, old_name, new_name);
        }
    }
}

impl Protocol for Bus {
    fn get_type(&self) -> SocketType {
        SocketType::Bus
    }

    fn add_pipe(&mut self, tok: mio::Token, pipe: Pipe) -> io::Result<()> {
        let res = self.body.add_pipe(tok, pipe);

        if res.is_ok() {
            self.apply(|s, body| s.on_pipe_added(body, tok));
        }

        res
     }

    fn remove_pipe(&mut self, tok: mio::Token) -> Option<Pipe> {
        let pipe = self.body.remove_pipe(tok);

        if pipe.is_some() {
            self.apply(|s, body| s.on_pipe_removed(body, tok));
        }

        pipe
    }

    fn open_pipe(&mut self, event_loop: &mut EventLoop, tok: mio::Token) {
        self.apply(|s, body| s.open_pipe(body, event_loop, tok));
    }

    fn on_pipe_opened(&mut self, event_loop: &mut EventLoop, tok: mio::Token) {
        self.apply(|s, body| s.on_pipe_opened(body, event_loop, tok));
    }

    fn send(&mut self, event_loop: &mut EventLoop, msg: Message, timeout: Timeout) {
        let (raw_msg, pipe_id) = encode(msg);
        let origin = pipe_id.map(|id| mio::Token(id as usize));

        self.apply(|s, body| s.send(body, event_loop, Rc::new(raw_msg), origin, timeout));
    }

    fn on_send_done(&mut self, event_loop: &mut EventLoop, tok: mio::Token) {
        self.apply(|s, body| s.on_send_done(body, event_loop, tok));
    }

    fn on_send_timeout(&mut self, event_loop: &mut EventLoop) {
        self.apply(|s, body| s.on_send_timeout(body, event_loop));
    }

    fn has_pending_send(&self) -> bool {
        false
    }

    fn recv(&mut self, event_loop: &mut EventLoop, timeout: Timeout) {
        self.apply(|s, body| s.recv(body, event_loop, timeout));
    }

    fn on_recv_done(&mut self, event_loop: &mut EventLoop, tok: mio::Token, raw_msg: Message) {
        let msg = decode(raw_msg, tok);

        self.apply(|s, body| s.on_recv_done(body, event_loop, tok, msg));
    }

    fn on_recv_timeout(&mut self, _: &mut EventLoop) {
        self.apply(|s, body| s.on_recv_timeout(body));
    }

    fn ready(&mut self, event_loop: &mut EventLoop, tok: mio::Token, events: mio::EventSet) {
        self.apply(|s, body| s.ready(body, event_loop, tok, events));
    }

    fn destroy(&mut self, event_loop: &mut EventLoop) {
        self.body.destroy_pipes(event_loop);
    }
}

impl State {
    fn name(&self) -> &'static str {
        match *self {
            State::Idle           => "Idle",
            State::Receiving(_,_) => "Receiving",
            State::RecvOnHold(_)  => "RecvOnHold"
        }
    }

    fn on_pipe_added(self, _: &mut Body, _: mio::Token) -> State {
        self
    }

    fn on_pipe_removed(self, _: &mut Body, tok: mio::Token) -> State {
        match self {
            State::Receiving(token, timeout) => {
                if tok == token {
                    State::RecvOnHold(timeout)
                } else {
                    State::Receiving(token, timeout)
                }
            },
            other => other
        }
    }

    fn open_pipe(self, body: &mut Body, event_loop: &mut EventLoop, tok: mio::Token) -> State {
        body.open_pipe(event_loop, tok);

        self
    }

    fn on_pipe_opened(self, body: &mut Body, event_loop: &mut EventLoop, tok: mio::Token) -> State {
        body.on_pipe_opened(event_loop, tok);

        match self {
            State::RecvOnHold(t) => State::Idle.recv(body, event_loop, t),
            other                => other
        }
    }

    fn send(self, body: &mut Body, event_loop: &mut EventLoop, msg: Rc<Message>, origin: Option<mio::Token>, timeout: Option<mio::Timeout>) -> State {
        body.send(event_loop, msg, origin, timeout);

        State::Idle
    }

    fn on_send_done(self, body: &mut Body, event_loop: &mut EventLoop, tok: mio::Token) -> State {
        body.on_send_done(event_loop, tok);

        self
    }

    fn on_send_timeout(self, _: &mut Body, _: &mut EventLoop) -> State {
        self
    }

    fn recv(self, body: &mut Body, event_loop: &mut EventLoop, timeout: Option<mio::Timeout>) -> State {
        if let Some(tok) = body.recv_from(event_loop) {
            State::Receiving(tok, timeout)
        } else {
            State::RecvOnHold(timeout)
        }
    }

    fn on_recv_done(self, body: &mut Body, event_loop: &mut EventLoop, tok: mio::Token, msg: Message) -> State {
        match self {
            State::Receiving(token, timeout) => {
                if tok == token {
                    body.on_recv_done(event_loop, msg, timeout);
                    State::Idle
                } else {
                    body.on_recv_done_late(event_loop, tok);
                    State::Receiving(token, timeout)
                }
            }
            other => {
                body.on_recv_done_late(event_loop, tok);
                other
            }
        }
    }

    fn on_recv_timeout(self, body: &mut Body) -> State {
        body.on_recv_timeout();

        State::Idle
    }

    fn ready(self, body: &mut Body, event_loop: &mut EventLoop, tok: mio::Token, events: mio::EventSet) -> State {
        body.ready(event_loop, tok, events);

        match self {
            State::RecvOnHold(t) => State::Idle.recv(body, event_loop, t),
            other                => other
        }
    }
}

impl Body {

    fn send(&mut self, event_loop: &mut EventLoop, msg: Rc<Message>, origin: Option<mio::Token>, timeout: Timeout) {
        if let Some(excluded) = origin {
            self.send_all_but(event_loop, msg, timeout, excluded);
        } else {
            self.send_all(event_loop, msg, timeout);
        }
    }

}

impl WithNotify for Body {
    fn get_notify_sender(&self) -> &Sender<SocketNotify> {
        &self.notify_sender
    }
}

impl WithPipes for Body {
    fn get_pipes(&self) -> &HashMap<mio::Token, Pipe> {
        &self.pipes
    }

    fn get_pipes_mut(&mut self) -> &mut HashMap<mio::Token, Pipe> {
        &mut self.pipes
    }
}

impl WithFairQueue for Body {
    fn get_fair_queue(&self) -> &PrioList {
        &self.fq
    }

    fn get_fair_queue_mut(&mut self) -> &mut PrioList {
        &mut self.fq
    }
}

impl WithBroadcast for Body {
}

fn decode(raw_msg: Message, pipe_id: mio::Token) -> Message {
    let mut msg = raw_msg;
    let mut pipe_id_bytes: [u8; 4] = [0; 4];

    BigEndian::write_u32(&mut pipe_id_bytes[..], pipe_id.as_usize() as u32);

    msg.header.reserve(4);
    msg.header.extend_from_slice(&pipe_id_bytes);
    msg
}

fn encode(msg: Message) -> (Message, Option<u32>) {
    if msg.get_header().len() < 4 {
        return (msg, None);
    }

    let (mut header, body) = msg.explode();
    let remaining_header = header.split_off(4);
    let pipe_id = BigEndian::read_u32(&header);
    let raw_msg = Message::with_header_and_body(remaining_header, body);

    (raw_msg, Some(pipe_id))
}
