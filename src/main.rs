extern crate mqtt;
use mqtt::*;

fn main() -> () {
	let result = MqttConnection::connect("broker.mqttdashboard.com:1883", "koka58", None, None, None, false, 0);
	if let Ok(conn @ MqttConnection(_)) = result {
		conn.ping().unwrap();
		
		conn.subscribe(Topic::new("711128sensor"), QoS::QoS0).unwrap();
		conn.subscribe(Topic::new("PING"), QoS::QoS0).unwrap();
		conn.subscribe(Topic::new("koka58"), QoS::QoS0).unwrap();
		
		conn.publish(Message{
			topic : Topic::new("koka58"),
			payload : Payload::new_str("ololo-ololo"),
			qos : QoS::QoS0,
			retain : false
		}).unwrap();
		
		conn.listen(Box::new(|p| {
			match p {
				Packet::PUBLISH{ message : Message{ topic, payload, .. }, .. } => println!("RECEIVED MESSAGE {} -> {:?}", topic, payload.to_string()),
				_ => println!("RECEIVED {:?}", p)
			}
			Ok(())
		})).unwrap();
	} else {
		println!("Can't connect to broker!")
	}
}