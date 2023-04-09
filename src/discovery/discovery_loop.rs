use std::net::{Ipv4Addr, UdpSocket};

use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::device;
use fs_protobuf_rust::compiled::mcfs::status;
use fs_protobuf_rust::compiled::mcfs::command;
use quick_protobuf::deserialize_from_slice;
use quick_protobuf::{serialize_into_vec};

pub fn begin() {
    let mcast_group: Ipv4Addr = "224.0.0.3".parse().unwrap();
    let port: u16 = 6000;
    let any = "0.0.0.0".parse().unwrap();

    let socket = UdpSocket::bind((any, port)).expect("Could not bind client socket");
    socket.set_multicast_loop_v4(false).expect("set_multicast_loop_v4 call failed");
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



    let mut buffer = [0u8; 1600];

    loop {
        let result = socket.recv_from(&mut buffer);
        match result {
            Ok((_size, src)) => {
                // TODO: log discovery message
                if let Ok(core::Message {
                    content: core::mod_Message::OneOfcontent::command(
                        command::Command {
                            command: command::mod_Command::OneOfcommand::device_discovery(..)
                        }
                    ),..}
                ) = deserialize_from_slice(&buffer) {
                    println!("Received discovery message from {}", src);
                    let _result = socket.send_to(&response_serialized, &(mcast_group, port));
                } 
            }
            Err(_e) => {
                // TODO: log error
            }
        }
    }
}