use std::io;
use proto::*;

pub trait Writable {
	fn write_to(&self, writer : & mut io::Write) -> Result<usize, io::Error>;
}

impl Writable for Packet{
	fn write_to(&self, writer : & mut io::Write) -> Result<usize, io::Error> {
		let flags : u8 = match self {
			&Packet::PUBLISH { dup, ref message, .. } => {
				let mut flags = 0;
				if message.retain {flags += 1};
				flags += (message.qos as u8) << 1;
				if dup { flags += 0b1000 };
				flags
			},
			&Packet::PUBREL{..} => 0b10,
			&Packet::SUBSCRIBE{..} => 0b10,
			&Packet::UNSUBSCRIBE{..} => 0b10,
			_ => 0
		};
		
		let packet_type : u8 = match self {
			&Packet::CONNECT{..} => 1,
			&Packet::CONNACK{..} => 2,
			&Packet::PUBLISH{..} => 3,
			&Packet::PUBACK{..} => 4,
			&Packet::PUBREC{..} => 5,
			&Packet::PUBREL{..} => 6,
			&Packet::PUBCOMP{..} => 7,
			&Packet::SUBSCRIBE{..} => 8,
			&Packet::SUBACK{..} => 9,
			&Packet::UNSUBSCRIBE{..} => 10,
			&Packet::UNSUBACK{..} => 11,
			&Packet::PINGREQ => 12,
			&Packet::PINGRESP => 13,
			&Packet::DISCONNECT => 14
		};
		
		let var_hdr = match self {
			&Packet::CONNECT{
				ref username,
				ref password,
				ref will,
				clean_session,
				keep_alive,
				..
			} => {
				let mut vec = len_str("MQTT");
				vec.push(0x04);
				
				let mut flags : u8 = 0;
				if clean_session { flags += 2 };
				
				flags += match will {
					&Some(ref will) => {
						let mut flags = 4;
						flags += (will.qos as u8) << 3;
						if will.retain { flags += 32; }
						flags
					},
					&None => 0 
				};
				
				if password.is_some() { flags += 64 };
				if username.is_some() { flags += 128 };
				
				vec.push(flags);
				vec.extend(&big_endian(keep_alive));
				Some(vec)
			},
			&Packet::CONNACK{session_present, return_code} => {
				let mut vec = vec!();
				let mut flags = 0;
				if session_present {flags += 1};
				vec.push(flags);
				vec.push(return_code as u8); 
				Some(vec)
			},
			&Packet::PUBLISH{ref message, ref packet_id, ..} => {
				let mut vec = vec!();
				vec.extend(len_str(&message.topic.0));
				match (message.qos, packet_id) {
					(QoS::QoS0, _) => {},
					(_, &Some(ref pid)) => vec.extend(&big_endian(pid.0)),
					_ => {}
				}
				Some(vec)
			},
			&Packet::PUBACK{ref packet_id} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			&Packet::PUBREC{ref packet_id} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			&Packet::PUBREL{ref packet_id} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			&Packet::PUBCOMP{ref packet_id} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			&Packet::SUBSCRIBE{ref packet_id, ..} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			&Packet::SUBACK{ref packet_id, ..} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			&Packet::UNSUBSCRIBE{ref packet_id, ..} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			&Packet::UNSUBACK{ref packet_id} => {
				let mut vec = vec!();
				vec.extend(&big_endian(packet_id.0));
				Some(vec)
			},
			_ => None
		};
		
		let payload = match self {
			&Packet::CONNECT{
				ref username,
				ref password,
				ref will,
				ref client_id,
				..
			} => {
				let mut vec = vec!();
				vec.extend(len_str(client_id));
				
				match will {
					&Some(Message{
						ref topic,
						ref payload,
						..
					}) => {
						vec.extend(len_str(&topic.0));
						vec.extend(len_arr(&payload.0));
					},
					_ => {}
				}
				
				if let &Some(ref username) = username {
					vec.extend(len_str(username));
				}
				
				if let &Some(ref password) = password {
					vec.extend(len_arr(password));
				}
				
				Some(vec)
			},
			&Packet::PUBLISH{ref message, ..} => {
				let mut vec = vec!();
				vec.extend(&message.payload.0);
				Some(vec)
			},
			&Packet::SUBSCRIBE{ref topic_filters, ..} => {
				let mut vec = vec!();
				for filter in topic_filters {
					vec.extend(len_str(&(filter.0).0));
					vec.push(filter.1 as u8);					
				}
				Some(vec)
			},
			&Packet::SUBACK{ref return_codes, ..} => {
				let mut vec = vec!();
				for code in return_codes {
					vec.push(*code as u8);					
				}
				Some(vec)
			},
			&Packet::UNSUBSCRIBE{ref topic_filters, ..} => {
				let mut vec = vec!();
				for filter in topic_filters {
					vec.extend(len_str(&filter.0));
				}
				Some(vec)
			},
			_ => None
		};
		
		let mut bytes_written = 0;
		bytes_written += try!(writer.write(&[(packet_type << 4) | flags]));		
		bytes_written += try!(writer.write(&var_length(var_hdr.as_ref().map_or(0, |it| it.len()) + payload.as_ref().map_or(0, |it| it.len()))));
		
		if let Some(ref var_hdr) = var_hdr {
			bytes_written += try!(writer.write(&var_hdr));
		}
		
		if let Some(ref payload) = payload {
			bytes_written += try!(writer.write(payload));
		}
		
		Ok(bytes_written)
	}
	
}

fn len_arr(str : &[u8]) -> Vec<u8> {
	let len = big_endian(str.len() as u16);
	let mut vec = vec!(len[0], len[1]);
	vec.extend(str);
	vec
} 

fn len_str(str : &str) -> Vec<u8> {
	let len = big_endian(str.len() as u16);
	let mut vec = vec!(len[0], len[1]);
	vec.extend(str.as_bytes());
	vec
} 

fn big_endian (i : u16) -> [u8;2] {
	[(i & 0xFF00) as u8, (i & 0x00FF) as u8]
}

fn var_length (i : usize) -> Vec<u8> {
	let mut buf : Vec<u8> = vec!();
	let mut x = i;
	loop {
		let mut enc = x % 128;
		x = x / 128;
		if x > 0 { enc = enc | 128; }
		buf.push(enc as u8);
		if x == 0 { break };	
	}
	buf
}