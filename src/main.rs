extern crate mqtt;

use std::net::TcpStream;
use mqtt::*;

fn main() -> () {
	let mut stream = TcpStream::connect("broker.mqttdashboard.com:1883").unwrap();
	connect(&mut stream, "koka58", None, None, None, false, 0).unwrap();
	/*ping(&mut stream).unwrap();
	publish(&mut stream, Message{
		topic : Topic("koka58".to_string()),
		payload : "ololo-ololo".to_string().into_bytes(),
		qos : QoS::QoS0,
		retain : false
	}).unwrap();*/
	subscribe(&mut stream, Topic("40CCF15".to_string()), QoS::QoS0).unwrap();
	disconnect(&mut stream).unwrap();
}