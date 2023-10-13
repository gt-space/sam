use std::borrow::Cow;
use fs_protobuf_rust::compiled::google::protobuf::Timestamp;
use fs_protobuf_rust::compiled::mcfs::core;
use fs_protobuf_rust::compiled::mcfs::data;
use fs_protobuf_rust::compiled::mcfs::board;
use quick_protobuf::serialize_into_vec;
use rand::{distributions::Uniform, Rng};
use crate::adc;


pub fn data_message_formation(measurement: adc::Measurement, data: Vec<f64>) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 20000);
    let mut node_data: Vec<data::ChannelData> = Vec::new();
    for node_id in 1..2 {
    // let mut start = 1;
    // if measurement == adc::Measurement::Tc2 {
    //     start = 2;
    // }
    //for node_id in start..(data.len() + 1) as u32 {
        let offsets: Vec<u32> = (0..5).map(|_| rng.sample(&range)).collect();
        //let data: Vec<f32> = (0..5).map(|_| rng.sample(&range) as f32 * 1.2).collect();

        node_data.push(generate_node_data(offsets, data.clone(), node_id, measurement.clone()));
    }

    let data = data::Data {
        channel_data: node_data,
    };

    let data_message = core::Message {
        timestamp: Some(Timestamp { seconds: 9, nanos: 100 }),
        board_id: 1,
        content: core::mod_Message::OneOfcontent::data(data)
    };

    //println!("{:?}", data_message);

    let data_serialized = serialize_into_vec(&data_message).expect("Cannot serialize `data`");
    data_serialized
}

fn generate_node_data(offsets: Vec<u32>, data: Vec<f64>, node_id: u32, measurement: adc::Measurement) -> data::ChannelData<'static> {
    let node = board::ChannelIdentifier {
        board_id: 1,
        channel_type: measurement_to_channel_type(node_id, measurement).unwrap(),
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

fn measurement_to_channel_type(node_id: u32, measurement: adc::Measurement) -> Option<board::ChannelType> {
    match (node_id, measurement) {
        (_, adc::Measurement::CurrentLoopPt) => Some(board::ChannelType::CURRENT_LOOP),
        (_, adc::Measurement::VValve) => Some(board::ChannelType::VALVE_VOLTAGE),
        (_, adc::Measurement::IValve) => Some(board::ChannelType::VALVE_CURRENT),
        (1, adc::Measurement::VPower) => Some(board::ChannelType::RAIL_24V),
        (2, adc::Measurement::VPower) => Some(board::ChannelType::RAIL_5V5),
        (3, adc::Measurement::VPower) => Some(board::ChannelType::RAIL_5V), // Digital
        (4, adc::Measurement::VPower) => Some(board::ChannelType::RAIL_5V), // Analog 
        (5, adc::Measurement::VPower) => Some(board::ChannelType::RAIL_3V3),
        (1, adc::Measurement::IPower) => Some(board::ChannelType::CURRENT_LOOP), // 24V
        (2, adc::Measurement::IPower) => Some(board::ChannelType::CURRENT_LOOP), // 5V
        (_, adc::Measurement::DiffSensors) => Some(board::ChannelType::DIFFERENTIAL_SIGNAL),
        (_, adc::Measurement::Rtd) => Some(board::ChannelType::RTD),
        (_, adc::Measurement::Tc1) => Some(board::ChannelType::TC),
        (_, adc::Measurement::Tc2) => Some(board::ChannelType::TC),
        (_, _) => None,
    }
}
