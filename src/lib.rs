/*
 * This file is part of Tokio ZMQ.
 *
 * Copyright © 2017 Riley Trautman
 *
 * Tokio ZMQ is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tokio ZMQ is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tokio ZMQ.  If not, see <http://www.gnu.org/licenses/>.
 */

#![feature(try_from)]

//! Tokio ZMQ, bringing ZeroMQ to the Tokio event loop
//!
//! This crate provides Streams, Sinks, and Futures for ZeroMQ Sockets, which deal in structures
//! caled Multiparts. Currently, a Multipart is a simple wrapper around `VecDeque<zmq::Message>`,
//! but in the future this will be represented as a wrapper around `VecDeque<S: zmq::Sendable>`
//! with the zmq 0.9 release.
//!
//! # Creating a socket
//!
//! To get a new socket, you must invoke the Socket builder. The Socket Builder can output a
//! 'raw' Socket, or any specific kind of socket, such as Rep, Req, etc. The result of the builder
//! can be any compatable kind of socket, so specifiying a type is important.
//!
//! Once you have a socket, if it implements `StreamSocket`, you can use the socket's `.stream()`, if
//! it implements `SinkSocket`, you can use the socket's `.sink()`, and if it implements
//! `FutureSocket`, you can use the `send` and `recv` methods.
//!
//! Without further ado, creating and using a socket:
//!
//! ```rust
//! #![feature(try_from)]
//!
//! extern crate zmq;
//! extern crate futures_util;
//! extern crate tokio;
//! extern crate tokio_zmq;
//!
//! use std::convert::TryInto;
//! use std::sync::Arc;
//!
//! use futures_util::{FutureExt, StreamExt};
//! use tokio_zmq::prelude::*;
//! use tokio_zmq::{Socket, Pub, Sub, Error};
//!
//! fn run() -> Result<(), Error> {
//!     // Create a new ZeroMQ Context. This context will be used to create all the sockets.
//!     let context = Arc::new(zmq::Context::new());
//!
//!     // Create our two sockets using the Socket builder pattern.
//!     // Note that the variable is named zpub, since pub is a keyword
//!     let zpub: Pub = Socket::builder(Arc::clone(&context))
//!         .bind("tcp://*:5561")
//!         .try_into()?;
//!
//!     let sub: Sub = Socket::builder(context)
//!         .bind("tcp://*:5562")
//!         .filter(b"")
//!         .try_into()?;
//!
//!     // Create our simple server. This forwards messages from the Subscriber socket to the
//!     // Publisher socket, and prints them as they go by.
//!     let runner = sub.stream()
//!         .map(|multipart| {
//!             for msg in &multipart {
//!                 if let Some(msg) = msg.as_str() {
//!                     println!("Forwarding: {}", msg);
//!                 }
//!             }
//!             multipart
//!         })
//!         .forward(zpub.sink());
//!
//!     // To avoid an infinte doctest, the actual core.run is commented out.
//!     // tokio::runtime::run2(runner.map(|_| ()).or_else(|e| {
//!     //     println!("Error: {}", e);
//!     // })?;
//!     # let _ = runner;
//!     # Ok(())
//! }
//!
//! # fn main() {
//! #     run().unwrap();
//! # }
//! ```

extern crate futures_core;
extern crate futures_sink;
extern crate futures_util;
#[macro_use]
extern crate log;
extern crate mio;
extern crate tokio_file_unix;
extern crate tokio_reactor;
extern crate tokio_timer_futures2 as tokio_timer;
#[macro_use]
extern crate tokio_zmq_derive;
extern crate zmq;

mod error;
mod message;
pub mod async;
pub mod socket;
pub mod file;
pub mod prelude;

pub use self::error::Error;
pub use self::message::Multipart;
pub use self::socket::Socket;
pub use self::socket::types::{Dealer, Pair, Pub, Pull, Push, Rep, Req, Router, Sub, Xpub, Xsub};
