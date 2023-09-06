use std::net::{Ipv4Addr, UdpSocket};
use std::time::Duration;

use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::device;
use fs_protobuf_rust::compiled::mcfs::status;
use quick_protobuf::{serialize_into_vec};

pub fn begin(broadcast_period: Duration) {
    let mcast_group: Ipv4Addr = "224.0.0.3".parse().unwrap();
    let port: u16 = 6000;
    let any = "0.0.0.0".parse().unwrap();

    let socket = UdpSocket::bind((any, port)).expect("Could not bind client socket");
    socket.set_multicast_loop_v4(false).expect("set_multicast_loop_v4 call failed");
    socket.set_read_timeout(Some(Duration::new(10, 0))).expect("set_read_timeout call failed");
    socket
        .join_multicast_v4(&mcast_group, &any)
        .expect("Could not join multicast group");

    let response = core::Message {
        timestamp: None,
        board_id: 1,
        content: core::mod_Message::OneOfcontent::status(status::Status {
            status_message: std::borrow::Cow::Borrowed(""),
            status: status::mod_Status::OneOfstatus::device_info(status::DeviceInfo {
                board_id: 1, 
                device_type: device::DeviceType::SAM 
            })
        }),
    };

    let response_serialized = serialize_into_vec(&response).expect("Could not serialize discovery response");
    let mut last_broadcasted = std::time::Instant::now();


    loop {
        read_device_info(&socket);
        let time_since_last_broadcast = last_broadcasted.elapsed();
        if time_since_last_broadcast > broadcast_period {
            let _send_result = socket.send_to(&response_serialized, (mcast_group, port));
            last_broadcasted = std::time::Instant::now();
            println!("Sent device info");
        }
    }
}

/*
 * Listen on the socket for a device info message
 * The SAM boards currently do not store any information on other devices
 * so we can just ignore the message
 * 
 * The recv-from call will timeout and return to the discovery loop
 */
pub fn read_device_info(socket: &UdpSocket) {
    let mut buffer = [0u8; 1600];
    let _result = socket.recv_from(&mut buffer);
}