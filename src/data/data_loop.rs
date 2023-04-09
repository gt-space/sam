use std::thread::sleep;
use std::time::{Duration, Instant};

use fs_protobuf_rust::compiled::google::protobuf::Timestamp;
use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::data;
use fs_protobuf_rust::compiled::mcfs::device;
use quick_protobuf::{serialize_into_vec};
use rand::{distributions::Uniform, Rng};
use std::borrow::Cow;

use std::net::UdpSocket;

pub fn begin(frequency: u64) {
    let socket = UdpSocket::bind(("0.0.0.0", 4573)).expect("Could not bind client socket");
    let interval = Duration::from_micros(1000000 / frequency);
    let mut next_time = Instant::now() + interval;

    loop {
        let data_serialized = data_message_formation();

        socket
            .send_to(&data_serialized, ("224.0.0.7", 4573))
            .expect("couldn't send data");

        sleep(next_time - Instant::now());
        next_time += interval;
    }
}

fn data_message_formation() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 20000);
    let mut node_data: Vec<data::NodeData> = Vec::new();
    for node_id in 1..2 {
        let offsets: Vec<u32> = (0..5).map(|_| rng.sample(&range)).collect();
        let data: Vec<f32> = (0..5).map(|_| rng.sample(&range) as f32 * 1.2).collect();

        node_data.push(generate_node_data(offsets, data, node_id));
    }

    let data = data::Data {
        node_data: node_data,
    };

    let data_message = core::Message {
        timestamp: Some(Timestamp { seconds: 9, nanos: 100 }),
        board_id: 1,
        content: core::mod_Message::OneOfcontent::data(data)
    };

    let data_serialized = serialize_into_vec(&data_message).expect("Cannot serialize `data`");
    data_serialized
}

fn generate_node_data(offsets: Vec<u32>, data: Vec<f32>, node_id: u32) -> data::NodeData<'static> {
    let node = device::NodeIdentifier {
        board_id: 1,
        channel: device::Channel::GPIO,
        node_id: node_id,
    };

    let node_data = data::NodeData {
        timestamp: Some(Timestamp {
            seconds: 9,
            nanos: 100,
        }),
        node: Some(node),
        micros_offsets: offsets,
        data_points: data::mod_NodeData::OneOfdata_points::f32_array(data::F32Array {data: Cow::from(data)})
    };

    return node_data;
}
