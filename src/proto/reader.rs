use std::io;
use num::FromPrimitive;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;
use proto::*;

pub trait Readable {
	fn read_from(reader : &mut io::Read) -> Result<Packet, Box<Error>>;
}

#[derive(Copy, Clone, Debug)]
pub enum ReaderError {
	NoLSB,
	NoMSB,
	BadQoS(u8),
	BadReturnCode(u8),
	IncompleteMessage,
	UnknownMessage(u8)
}

impl Display for ReaderError {
	fn fmt(&self, fmt : &mut Formatter) -> Result<(), fmt::Error> {
		fmt.write_str(self.description())
	}
}

impl Error for ReaderError {
	fn description(&self) -> &str {
		match self {
			&ReaderError::NoLSB => "No LSB error",
			&ReaderError::NoMSB => "No MSB error",
			&ReaderError::BadQoS(..) => "Bad QoS",
			&ReaderError::BadReturnCode(..) => "Bad return code",
			&ReaderError::IncompleteMessage => "Incomplete message",
			&ReaderError::UnknownMessage(..) => "Unknown message"
		}
	}
}

impl Readable for Packet {
	fn read_from(reader : &mut io::Read) -> Result<Packet, Box<Error>> {
		let first_byte = try!(read_byte(reader));
		let packet_type = (first_byte & 0xF0) >> 4;
		let flags = first_byte & 0x0F; 
		
		let remaining_length = try!(var_length(reader));

		let mut remaining = try!(read_vec(reader, remaining_length)).into_iter().peekable();
		
		println!("READ PACKET type={} flags={} remaining={:?}", packet_type, flags, remaining_length);
		
		let result = match packet_type {
			1 => {
				skip(&mut remaining, 7);
				let flags = try!(remaining.next().ok_or(ReaderError::IncompleteMessage));
				
				let clean_session = (flags & 2) == 1;
				let has_will = (flags & 4) == 1;
				let will_qos = ((flags & 16) << 1) + (flags & 8); 
				let will_retain = (flags & 32) == 1;
				let has_password = (flags & 64) == 1;
				let has_username = (flags & 128) == 1;
			
				let keep_alive = try!(big_endian(&mut remaining));
				let client_id = try!(len_str(&mut remaining));
				
				let will = if has_will {
					let topic = try!(len_str(&mut remaining));
					let payload = try!(len_arr(&mut remaining));
					Some(Message{
						topic : Topic(topic),
						payload : payload,
						qos : try!(QoS::from_u8(will_qos).ok_or(ReaderError::BadQoS(will_qos))),
						retain : will_retain
					})
				} else {
					None
				};
				
				let username = if has_username {
					Some(try!(len_str(&mut remaining)))
				} else {
					None
				};
				
				let password = if has_password {
					Some(try!(len_arr(&mut remaining)))
				} else {
					None
				};
				
				Ok(Packet::CONNECT{ 
					client_id : client_id,
					username : username,
					password : password,
					will : will,
					clean_session : clean_session,
					keep_alive : keep_alive
				})
			},
			2 => {
				let flags = try!(remaining.next().ok_or(ReaderError::IncompleteMessage));
				let return_code = try!(remaining.next().ok_or(ReaderError::IncompleteMessage));
				Ok(Packet::CONNACK{ session_present : flags & 1 == 1, return_code : try!(ConnectReturnCode::from_u8(return_code).ok_or(ReaderError::BadReturnCode(return_code))) })
			},
			3 => {
				let dup = (flags & 8) == 1;
				let retain = (flags & 1) == 1;
				let qos_byte = ((flags & 4) << 1) + (flags & 2);
				let qos = try!(QoS::from_u8(qos_byte).ok_or(ReaderError::BadQoS(qos_byte)));
				let topic = try!(len_str(&mut remaining));
				
				let packet_id = match qos {
					QoS::QoS0 => None,
					_ => Some(try!(big_endian(&mut remaining)))
				};
				
				let offset = 2 + topic.len() + match packet_id {
					Some(..) => 2,
					None => 0
				};
				
				let payload = try!(arr(&mut remaining, remaining_length - offset));
				Ok(Packet::PUBLISH{
					dup : dup,
					message : Message {
						topic : Topic(topic),
						payload : payload,
						qos : qos,
						retain : retain
					},
					packet_id : packet_id.map(|pid| PacketId(pid))
				})
			},
			4 => {
				let packet_id = try!(big_endian(&mut remaining));
				Ok(Packet::PUBACK{ packet_id : PacketId(packet_id) })
			},
			5 => {
				let packet_id = try!(big_endian(&mut remaining));
				Ok(Packet::PUBREC{ packet_id : PacketId(packet_id) })
			},
			6 => {
				let packet_id = try!(big_endian(&mut remaining));
				Ok(Packet::PUBREL{ packet_id : PacketId(packet_id) })
			},
			7 => {
				let packet_id = try!(big_endian(&mut remaining));
				Ok(Packet::PUBCOMP{ packet_id : PacketId(packet_id) })
			},
			8 => {
				let packet_id = try!(big_endian(&mut remaining));
				let mut vec = vec!();
				while remaining.peek().is_some() {
					let topic = try!(len_str(&mut remaining));
					let qos = try!(remaining.next().ok_or(ReaderError::IncompleteMessage));
					vec.push((Topic(topic), try!(QoS::from_u8(qos).ok_or(ReaderError::BadQoS(qos)))));					
				}
				Ok(Packet::SUBSCRIBE{ packet_id : PacketId(packet_id), topic_filters : vec })
			},
			9 => {
				let packet_id = try!(big_endian(&mut remaining));
				let mut vec = vec!();
				while remaining.peek().is_some() {
					let code = try!(remaining.next().ok_or(ReaderError::IncompleteMessage));
					vec.push(try!(SubackReturnCode::from_u8(code).ok_or(ReaderError::BadReturnCode(code))));					
				}
				Ok(Packet::SUBACK{ packet_id : PacketId(packet_id), return_codes : vec })
			},
			10 => {
				let packet_id = try!(big_endian(&mut remaining));
				let mut vec = vec!();
				while remaining.peek().is_some() {
					let topic = try!(len_str(&mut remaining));
					vec.push(Topic(topic));					
				}
				Ok(Packet::UNSUBSCRIBE{ packet_id : PacketId(packet_id), topic_filters : vec })
			},
			11 => {
				let packet_id = try!(big_endian(&mut remaining));
				Ok(Packet::UNSUBACK{ packet_id : PacketId(packet_id) })
			},
			12 => Ok(Packet::PINGREQ),
			13 => Ok(Packet::PINGRESP),
			14 => Ok(Packet::DISCONNECT),
			
			_ => Err(ReaderError::UnknownMessage(packet_type))
		};
		
		Ok(try!(result))
	}
}

fn skip(buf : &mut Iterator<Item=u8>, num : usize) {
	for _ in 0..num {
		buf.next();
	}
}


fn len_str(buf : &mut Iterator<Item=u8>) -> Result<String, Box<Error>> {
	Ok(try!(String::from_utf8(try!(len_arr(buf)))))
}

fn len_arr(buf : &mut Iterator<Item=u8>) -> Result<Vec<u8>, Box<Error>> {
	let len = try!(big_endian(buf)) as usize;
	arr(buf, len)
}

fn arr(buf : &mut Iterator<Item=u8>, size : usize) -> Result<Vec<u8>, Box<Error>> {
	let mut len = size;
	let mut vec = vec!();
	while len > 0 {
		vec.push(try!(buf.next().ok_or(ReaderError::IncompleteMessage)));
		len -= 1;
	}
	Ok(vec)
}

fn big_endian(buf : &mut Iterator<Item=u8>) -> Result<u16, ReaderError> {
	let msb = try!(buf.next().ok_or(ReaderError::NoMSB));
	let lsb = try!(buf.next().ok_or(ReaderError::NoLSB));
	Ok(((msb as u16) << 8) + (lsb as u16))
}

fn read_byte(reader : &mut io::Read) -> Result<u8, Box<Error>> {
	let mut buf : [u8; 1] = [0;1];
	let bytes_read = try!(reader.read(&mut buf));
	if bytes_read == 0 {
		Err(Box::new(ReaderError::IncompleteMessage))
	} else {
		Ok(buf[0])
	}
}

fn read_vec(reader : &mut io::Read, length : usize) -> Result<Vec<u8>, Box<Error>> {
	let mut buf = vec!();
	for _ in 0..length {
		buf.push(0);
	}
	let bytes_read = try!(reader.read(&mut buf));
	if bytes_read < length {
		Err(Box::new(ReaderError::IncompleteMessage))
	} else {
		Ok(buf)
	}
}

fn var_length(reader : &mut io::Read) -> Result<usize, Box<Error>> {
	let mut multiplier = 1;
	let mut value : usize = 0;
	loop {
		let enc = try!(read_byte(reader));
		value += multiplier * (enc & 127) as usize;
		multiplier *= 128;
		
		if multiplier > 128*128*128 {
			return Err(Box::new(ReaderError::IncompleteMessage));
		} 
		
		if enc & 128 == 0 { break };	
	}
	Ok(value)
}