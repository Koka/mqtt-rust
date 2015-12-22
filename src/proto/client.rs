use proto::{PacketId};
use proto::writer::Writable;
use proto::reader::Readable;
use std::net::TcpStream;
use std::error::Error;
use std::fmt::Display;
use std::fmt;

pub use proto::{Message, Topic, QoS, Payload, Packet};

pub struct MqttConnection(TcpStream);

#[derive(Debug)]
pub struct MqttError(String);

impl Display for MqttError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MqttError: {}", self.0)
    }
}

impl Error for MqttError {
	fn description(&self) -> &str {
		"MQTT protocol error"
	}
	
	fn cause(&self) -> Option<&Error> { None }
}

impl MqttConnection {
	pub fn connect(url : &str, client_id : &str, username : Option<&str>, password : Option<&str>, last_will : Option<Message>, clean_session : bool, keep_alive : u16) -> Result<MqttConnection, Box<Error>> {
		let mut tcp = try!(TcpStream::connect(url));
		try!(Packet::CONNECT {
			client_id : client_id.to_string(),
			username : username.map(|s| s.to_string()),
			password : password.map(|s| s.to_string().into_bytes()),
			will : last_will,
			clean_session : clean_session,
			keep_alive : keep_alive
		}.write_to(&mut tcp));
	
		match try!(Packet::read_from(&mut tcp)) {
			Packet::CONNACK{..} => Ok(MqttConnection(tcp)),
			_ => Err(Box::new(MqttError("Not CONNACK response".to_string())))
		}
	} 
	
	pub fn publish(&self, msg : Message) -> Result<(), Box<Error>> {
		let mut tcp = &self.0;
		/* TODO: do all the magic */
		let request = Packet::PUBLISH {
			dup : false,
			message : msg,
			packet_id : None
		};
		try!(request.write_to(&mut tcp));
		try!(Packet::read_from(&mut tcp));
		Ok(())
	} 
	
	pub fn ping(&self) -> Result<(), Box<Error>> {
		let mut tcp = &self.0;
		try!(Packet::PINGREQ.write_to(&mut tcp));
		Ok(())
	} 
	
	pub fn subscribe(&self, topic : Topic, qos : QoS) -> Result<(), Box<Error>> {
		let mut tcp = &self.0;
		try!(Packet::SUBSCRIBE{
			packet_id : PacketId(12345),
			topic_filters : vec!((topic, qos))
		}.write_to(&mut tcp));
		Ok(())
	} 
	
	pub fn listen(&self, callback: Box<Fn(Packet) -> Result<(), Box<Error>>> ) -> Result<(), Box<Error>> {
		println!("Listen!");
		loop {
			//TODO
			let mut tcp = &self.0;
			let packet = try!(Packet::read_from(&mut tcp));
			try!(callback(packet));
		}
	}
}

impl Drop for MqttConnection {
	fn drop(&mut self) {
        let mut tcp = &self.0;
		let _ = Packet::DISCONNECT.write_to(&mut tcp);
    }
}