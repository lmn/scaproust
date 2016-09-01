# scaproust <img src=albertine-like.jpg align=right width=200 height=200>

[![Linux build](https://travis-ci.org/blabaere/scaproust.svg?label=linux)](https://travis-ci.org/blabaere/scaproust)
[![Windows build](https://ci.appveyor.com/api/projects/status/kpqdm42mhlki39fq?svg=true)](https://ci.appveyor.com/project/blabaere/scaproust)
[![Clippy Linting Result](http://clippy.bashy.io/github/blabaere/scaproust/master/badge.svg)](http://clippy.bashy.io/github/blabaere/scaproust/master/log)

Scaproust is an implementation of the [nanomsg](http://nanomsg.org/index.html) "Scalability Protocols" in the [Rust programming language](http://www.rust-lang.org/).

**Experimental work !** For working stuff, please see [nanomsg-rs](https://github.com/blabaere/nanomsg.rs).  

[API Documentation](https://blabaere.github.io/scaproust/scaproust/index.html)

## Goals
* Support for all of nanomsg's protocols.
* Support for TCP and IPC transports.
* Idiomatic rust API first, mimic the original C API second.

## Maybe, later
* Nonblocking operations.
* Polling, on several sockets.
* Other transports (TLS, WebSockets).

## Non goals
* Ability to use a socket as a raw file descriptor with system level functions.

## Progress
- [ ] Features
  - [x] Socket
  - [ ] Device
  - [ ] Examples
  - [ ] Documentation
  - [x] User defined protocols
  - [ ] User defined transports

- [ ] Protocols
  - [x] PAIR
  - [x] BUS
  - [ ] REQREP
    - [ ] REQ
    - [ ] REQ resend
    - [ ] REQ prefetch replies
    - [ ] REP
  - [x] PUBSUB
    - [x] PUB
    - [x] SUB
    - [x] SUB subscription filter
  - [x] PIPELINE
    - [x] PUSH
    - [x] PULL
  - [ ] SURVEY
    - [ ] SURVEYOR
    - [ ] SURVEYOR deadline
    - [ ] RESPONDENT  

- [ ] Transports
  - [x] TCP
  - [ ] IPC (*nix only)
  - [ ] IPC (Windows)

- [ ] Socket options
  - [ ] Linger
  - [ ] Recv buffer size max
  - [x] Send timeout
  - [x] Recv timeout
  - [x] Reconnect interval
  - [ ] Reconnect interval max
  - [x] Send priority
  - [x] Recv priority
  - [ ] IPV4 only
  - [ ] Socket name

- [ ] Protocol options
  - [ ] REQ resend interval
  - [ ] SURVEYOR deadline
  - [x] SUB subscribe
  - [x] SUB unsubscribe

- [ ] Transport options
  - [ ] TCP no delay

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.