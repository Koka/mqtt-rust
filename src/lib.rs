#[macro_use] extern crate enum_primitive;
extern crate num;

mod proto;

use proto::{Packet, PacketId};
use proto::writer::Writable;
use proto::reader::Readable;
use std::net::TcpStream;
use std::error::Error;

pub use proto::{Message, Topic, QoS};

pub fn connect(tcp : & mut TcpStream, client_id : &str, username : Option<&str>, password : Option<&str>, last_will : Option<Message>, clean_session : bool, keep_alive : u16) -> Result<(), Box<Error>> {
	try!(Packet::CONNECT {
		client_id : client_id.to_string(),
		username : username.map(|s| s.to_string()),
		password : password.map(|s| s.to_string().into_bytes()),
		will : last_will,
		clean_session : clean_session,
		keep_alive : keep_alive
	}.write_to(tcp));
	match try!(Packet::read_from(tcp)) {
		Packet::CONNACK{..} => Ok(()),
		_ => panic!("Bad response")
	}
} 

pub fn publish(tcp : & mut TcpStream, msg : Message) -> Result<(), Box<Error>> {
	/* TODO: do all the magic */
	let request = Packet::PUBLISH {
		dup : false,
		message : msg,
		packet_id : None
	};
	try!(request.write_to(tcp));
	try!(Packet::read_from(tcp));
	Ok(())
} 

pub fn ping(tcp : & mut TcpStream) -> Result<(), Box<Error>> {
	try!(Packet::PINGREQ.write_to(tcp));
	match try!(Packet::read_from(tcp)) {
		Packet::PINGRESP => Ok(()),
		_ => panic!("Bad response")
	}
} 

pub fn subscribe(tcp : & mut TcpStream, topic : Topic, qos : QoS) -> Result<(), Box<Error>> {
	try!(Packet::SUBSCRIBE{
		packet_id : PacketId(12345),
		topic_filters : vec!((topic, qos))
	}.write_to(tcp));
	match try!(Packet::read_from(tcp)) {
		Packet::SUBACK{..} => Ok(()),
		_ => panic!("Bad response")
	}
} 

pub fn listen(tcp : & mut TcpStream) -> Result<(), Box<Error>> {
	println!("Listen!");
	loop {
		//TODO
		let packet = try!(Packet::read_from(tcp));
		println!("PACKET RECEIVED {:?}", packet);
		if let Packet::PUBLISH{ message : Message{ payload, .. }, .. } = packet {
			println!("\tPAYLOAD {:?}", String::from_utf8_lossy(&payload));
		}
	}
}

pub fn disconnect(tcp : & mut TcpStream) -> Result<(), Box<Error>> {
	try!(Packet::DISCONNECT.write_to(tcp));
	Ok(())
} 