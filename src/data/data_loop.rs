use std::thread::sleep;
use std::time::{Duration, Instant, UNIX_EPOCH};

use fs_protobuf_rust::compiled::mcfs::data;
use fs_protobuf_rust::compiled::mcfs::device;
use fs_protobuf_rust::compiled::google::protobuf::Timestamp;
use quick_protobuf::{serialize_into_vec, deserialize_from_slice};
use std::borrow::Cow;
use rand::{distributions::Uniform, Rng};

use std::net::UdpSocket;

pub fn begin(frequency: u64) {
    let interval = Duration::from_micros(1000000 / frequency);
    let mut next_time = Instant::now() + interval;
    
    loop {

        let data_serialized = data_message_formation();

        let socket = UdpSocket::bind("0.0.0.0:24013").expect("Error");
        socket.send_to(&data_serialized, "192.168.6.1:7201").expect("couldn't send data");

        sleep(next_time - Instant::now());
        next_time += interval;

    }
}


fn data_message_formation() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 20000000);
    let mut node_data: Vec<data::NodeData> = Vec::new();
    for node_id in 1..40 {
        let offsets: Vec<u32> = (0..10).map(|_| rng.sample(&range)).collect();
        let data: Vec<u32> = (0..10).map(|_| rng.sample(&range)).collect();

        node_data.push(generate_node_data(offsets, data, node_id));
    }

    let data = data::Data {
        node_data: node_data,
    };

    let data_serialized = serialize_into_vec(&data).expect("Cannot serialize `data`");
    data_serialized
}

fn generate_node_data(offsets: Vec<u32>, data: Vec<u32>, node_id: u32) -> data::NodeData<'static> {
    let node = device::NodeIdentifier {
        board_id: 10,
        channel: device::Channel::GPIO,
        node_id: node_id,
    };

    let node_data = data::NodeData {
        timestamp: Some(Timestamp { seconds: 9, nanos: 100 }),
        node: Some(node),
        micros_offsets: offsets,
        data_points: data::mod_NodeData::OneOfdata_points::u32_array(data::U32Array { data }),
    };

    return node_data;
}