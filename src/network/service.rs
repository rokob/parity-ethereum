#![allow(dead_code)] //TODO: remove this after everything is done

use std::thread::{self, JoinHandle};
use mio::*;
use network::{Error, ProtocolHandler};
use network::host::{Host, HostMessage, PeerId, PacketId, ProtocolId};

/// IO Service with networking
pub struct NetworkService {
	thread: Option<JoinHandle<()>>,
	host_channel: Sender<HostMessage>
}

impl NetworkService {
	/// Starts IO event loop
	pub fn start() -> Result<NetworkService, Error> {
		let mut event_loop = EventLoop::new().unwrap();
        let channel = event_loop.channel();
		let thread = thread::spawn(move || {
			Host::start(&mut event_loop).unwrap(); //TODO:
		});
		Ok(NetworkService {
			thread: Some(thread),
			host_channel: channel
		})
	}

	/// Send a message over the network. Normaly `HostIo::send` should be used. This can be used from non-io threads.
	pub fn send(&mut self, peer: &PeerId, packet_id: PacketId, protocol: ProtocolId, data: &[u8]) -> Result<(), Error> {
		try!(self.host_channel.send(HostMessage::Send {
			peer: *peer,
			packet_id: packet_id,
			protocol: protocol,
			data: data.to_vec()
		}));
		Ok(())
	}

	/// Regiter a new protocol handler with the event loop.
	pub fn register_protocol(&mut self, handler: Box<ProtocolHandler+Send>, protocol: ProtocolId, versions: &[u8]) -> Result<(), Error> {
		try!(self.host_channel.send(HostMessage::AddHandler {
			handler: handler,
			protocol: protocol,
			versions: versions.to_vec(),
		}));
		Ok(())
	}
}

impl Drop for NetworkService {
	fn drop(&mut self) {
		self.host_channel.send(HostMessage::Shutdown).unwrap();
		self.thread.take().unwrap().join().unwrap();
	}
}

