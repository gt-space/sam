use std::borrow::Cow;
use std::thread::sleep;
use std::time::{Duration, Instant};

use fs_protobuf_rust::compiled::google::protobuf::Timestamp;
use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::data;
use fs_protobuf_rust::compiled::mcfs::board;
use quick_protobuf::{serialize_into_vec};
use rand::{distributions::Uniform, Rng};
use std::net::{IpAddr, SocketAddr};

use std::net::UdpSocket;

use crate::adc;
use crate::discovery::get_ips;

// pub fn begin(frequency: u64, adc: &mut adc::ADC) {
//     let socket = UdpSocket::bind(("0.0.0.0", 4573)).expect("Could not bind client socket");
//     let interval = Duration::from_micros(1000000 / frequency);
//     let mut next_time = Instant::now() + interval;
//     let mut rng = rand::thread_rng();
//     let range = Uniform::new(0, 20000);
//     let ip_hashmap = get_ips(&["flight-computer-01"]);
    
//     // let data: Vec<i32> = (0..5).map(|_| rng.sample(&range) as i32).collect();
//     let socket_addr = SocketAddr::new(ip_hashmap.get("flight-computer-01").unwrap().unwrap(), 4573);

//     loop {
//         let data = test_read_all(adc);
//         let data_serialized = data_message_formation(data.clone());
        
//         socket
//             .send_to(&data_serialized, socket_addr)
//             .expect("couldn't send data");

//         sleep(next_time - Instant::now());
//         next_time += interval;
//     }
// }

pub fn data_message_formation(data: Vec<f64>) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 20000);
    let mut node_data: Vec<data::ChannelData> = Vec::new();
    for node_id in 1..2 {
        let offsets: Vec<u32> = (0..5).map(|_| rng.sample(&range)).collect();
        //let data: Vec<f32> = (0..5).map(|_| rng.sample(&range) as f32 * 1.2).collect();

        node_data.push(generate_node_data(offsets, data.clone(), node_id));
    }

    let data = data::Data {
        channel_data: node_data,
    };

    let data_message = core::Message {
        timestamp: Some(Timestamp { seconds: 9, nanos: 100 }),
        board_id: 1,
        content: core::mod_Message::OneOfcontent::data(data)
    };

    let data_serialized = serialize_into_vec(&data_message).expect("Cannot serialize `data`");
    data_serialized
}

fn generate_node_data(offsets: Vec<u32>, data: Vec<f64>, node_id: u32) -> data::ChannelData<'static> {
    let node = board::ChannelIdentifier {
        board_id: 1,
        channel_type: board::ChannelType::DIFFERENTIAL_SIGNAL,
        channel: node_id,
    };

    let node_data = data::ChannelData {
        timestamp: Some(Timestamp {
            seconds: 9,
            nanos: 100,
        }),
        channel: Some(node),
        micros_offsets: offsets,
        data_points: data::mod_ChannelData::OneOfdata_points::f64_array(data::F64Array {data: Cow::from(data)})
    };

    return node_data;
}
