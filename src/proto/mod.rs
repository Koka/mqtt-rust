pub mod writer;
pub mod reader;

enum_from_primitive! {
	#[derive(Copy, Clone, Debug)]
	pub enum QoS {
		QoS0 = 0,
		QoS1 = 1,
		QoS2 = 2
	}
}

enum_from_primitive! {
	#[derive(Copy, Clone, Debug)]
	pub enum ConnectReturnCode {
		Accepted = 0,
		RefusedProto = 1,
		RefusedId = 2,
		RefusedUnavailable = 3,
		RefusedLogin = 4,
		RefusedAuth = 5
	}
}

enum_from_primitive! {
	#[derive(Copy, Clone, Debug)]
	pub enum SubackReturnCode {
		QoS0 = 0,
		QoS1 = 1,
		QoS2 = 2,
		Failure = 0x80
	}
}

#[derive(Debug)]
pub struct Topic(pub String);

#[derive(Debug)]
pub struct PacketId(pub u16);

#[derive(Debug)]
pub struct Message {
	pub topic : Topic,
	pub payload : Vec<u8>,
	pub qos : QoS,
	pub retain : bool
}

#[derive(Debug)]
pub enum Packet {
	CONNECT {
		client_id : String,
		username : Option<String>,
		password : Option<Vec<u8>>,
		will : Option<Message>,
		clean_session : bool,
		keep_alive : u16
	},
	CONNACK {
		session_present : bool,
		return_code : ConnectReturnCode
	},
	PUBLISH {
		dup : bool,
		message : Message,
		packet_id : Option<PacketId>
	},
	PUBACK {
		packet_id : PacketId
	},
	PUBREC {
		packet_id : PacketId
	},
	PUBREL {
		packet_id : PacketId
	},
	PUBCOMP {
		packet_id : PacketId
	},
	SUBSCRIBE {
		packet_id : PacketId,
		topic_filters : Vec<(Topic, QoS)>
	},
	SUBACK {
		packet_id : PacketId,
		return_codes : Vec<SubackReturnCode>
	},
	UNSUBSCRIBE {
		packet_id : PacketId,
		topic_filters : Vec<Topic>
	},
	UNSUBACK {
		packet_id : PacketId
	},
	PINGREQ,
	PINGRESP,
	DISCONNECT
}

