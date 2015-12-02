extern crate mqtt;

use std::net::TcpStream;
use mqtt::*;

fn main() -> () {
	let mut stream = TcpStream::connect("broker.mqttdashboard.com:1883").unwrap();
	connect(&mut stream, "koka58", None, None, None, false, 0).unwrap();
	ping(&mut stream).unwrap();
	subscribe(&mut stream, Topic("711128sensor".to_string()), QoS::QoS0).unwrap();
	publish(&mut stream, Message{
		topic : Topic("koka58".to_string()),
		payload : "ololo-ololo".to_string().into_bytes(),
		qos : QoS::QoS0,
		retain : false
	}).unwrap();
	listen(&mut stream).unwrap();
	disconnect(&mut stream).unwrap();
}