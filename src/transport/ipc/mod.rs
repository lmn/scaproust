// Copyright 2016 Benoît Labaere (benoit.labaere@gmail.com)
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

mod stub;
mod send;
mod recv;
mod acceptor;

use std::io;
use std::path;

use mio_uds::{UnixListener, UnixStream};

use self::stub::IpcPipeStub;
use self::acceptor::IpcAcceptor;
use transport::Transport;
use transport::pipe::Pipe;
use transport::acceptor::Acceptor;
use transport::async::AsyncPipe;

pub struct Ipc;

impl Transport for Ipc {
    fn connect(&self, url: &str, pids: (u16, u16)) -> io::Result<Box<Pipe>> {
        let filename = path::Path::new(url);
        let stream = try!(UnixStream::connect(filename));
        let stub = IpcPipeStub::new(stream);
        let pipe = box AsyncPipe::new(stub, pids);

        Ok(pipe)
    }

    fn bind(&self, url: &str, pids: (u16, u16)) -> io::Result<Box<Acceptor>> {
        let filename = path::Path::new(url);
        let listener = try!(UnixListener::bind(filename));
        let acceptor = box IpcAcceptor::new(listener, pids);

        Ok(acceptor)
    }
}