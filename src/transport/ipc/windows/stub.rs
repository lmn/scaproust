// Copyright (c) 2015-2017 Contributors as noted in the AUTHORS file.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

use std::ops::Deref;
use std::rc::Rc;
use std::io;
use std::io::{Read, Write};

use mio;

use mio_named_pipes::NamedPipe;

use core::Message;
use transport::ipc::send::SendOperation;
use transport::ipc::recv::RecvOperation;
use transport::async::stub::*;
use io_error::*;

/*****************************************************************************/
/*                                                                           */
/* IpcPipeStub                                                               */
/*                                                                           */
/*****************************************************************************/

pub struct IpcPipeStub {
    server: bool,
    named_pipe: NamedPipe,
    recv_max_size: u64,
    send_operation: Option<SendOperation>,
    recv_operation: Option<RecvOperation>
}

impl Deref for IpcPipeStub {
    type Target = mio::Evented;
    fn deref(&self) -> &Self::Target {
        &self.named_pipe
    }
}

impl IpcPipeStub {
    pub fn new_server(named_pipe: NamedPipe, recv_max_size: u64) -> IpcPipeStub {
        IpcPipeStub {
            server: true,
            named_pipe: named_pipe,
            recv_max_size: recv_max_size,
            send_operation: None,
            recv_operation: None
        }
    }

    pub fn new_client(named_pipe: NamedPipe, recv_max_size: u64) -> IpcPipeStub {
        IpcPipeStub {
            server: false,
            named_pipe: named_pipe,
            recv_max_size: recv_max_size,
            send_operation: None,
            recv_operation: None
        }
    }

    fn run_send_operation(&mut self, mut send_operation: SendOperation) -> io::Result<bool> {
        if try!(send_operation.run(&mut self.named_pipe)) {
            Ok(true)
        } else {
            self.send_operation = Some(send_operation);
            Ok(false)
        }
    }

    fn run_recv_operation(&mut self, mut recv_operation: RecvOperation) -> io::Result<Option<Message>> {
        match try!(recv_operation.run(&mut self.named_pipe)) {
            Some(msg) => Ok(Some(msg)),
            None => {
                self.recv_operation = Some(recv_operation);
                Ok(None)
            }
        }
    }
}

impl Drop for IpcPipeStub {
    fn drop(&mut self) {
        let _ = self.named_pipe.disconnect();
    }
}

/*****************************************************************************/
/*                                                                           */
/* Sender for IpcPipeStub                                                    */
/*                                                                           */
/*****************************************************************************/

impl Sender for IpcPipeStub {
    fn start_send(&mut self, msg: Rc<Message>) -> io::Result<bool> {
        let send_operation = SendOperation::new(msg);

        self.run_send_operation(send_operation)
    }

    fn resume_send(&mut self) -> io::Result<bool> {
        if let Some(send_operation) = self.send_operation.take() {
            self.run_send_operation(send_operation)
        } else {
            Err(other_io_error("Cannot resume send: no pending operation"))
        }
    }

    fn has_pending_send(&self) -> bool {
        self.send_operation.is_some()
    }
}

/*****************************************************************************/
/*                                                                           */
/* Receiver for IpcPipeStub                                                  */
/*                                                                           */
/*****************************************************************************/

impl Receiver for IpcPipeStub {
    fn start_recv(&mut self) -> io::Result<Option<Message>> {
        let recv_operation = RecvOperation::new(self.recv_max_size);

        self.run_recv_operation(recv_operation)
    }

    fn resume_recv(&mut self) -> io::Result<Option<Message>> {
        if let Some(recv_operation) = self.recv_operation.take() {
            self.run_recv_operation(recv_operation)
        } else {
            Err(other_io_error("Cannot resume recv: no pending operation"))
        }
    }

    fn has_pending_recv(&self) -> bool {
        self.recv_operation.is_some()
    }
}

/*****************************************************************************/
/*                                                                           */
/* Handshake for IpcPipeStub                                                 */
/*                                                                           */
/*****************************************************************************/

impl Handshake for IpcPipeStub {
    fn send_handshake(&mut self, pids: (u16, u16)) -> io::Result<()> {
        send_and_check_handshake(&mut self.named_pipe, pids)
    }
    fn recv_handshake(&mut self, pids: (u16, u16)) -> io::Result<()> {
        recv_and_check_handshake(&mut self.named_pipe, pids)
    }
}

impl AsyncPipeStub for IpcPipeStub {
    #[cfg(windows)]
    fn read_and_write_void(&mut self) {
        let mut buffer: [u8; 0] = [0; 0];

        let _ = self.named_pipe.read(&mut buffer);
        let _ = self.named_pipe.write(&buffer);
    }

    #[cfg(windows)]
    fn registered(&mut self) {
        if self.server {
            let _ = self.named_pipe.connect();
        }
    }
}
