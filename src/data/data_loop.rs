use std::borrow::Cow;

use common::comm::DataMessage;
use common::comm::DataPoint;
use crate::adc;


pub fn serialize_data(data_points: &Vec<DataPoint>) -> Result<Vec<u8>, postcard::Error> {
    let data_message = DataMessage::Sam(Cow::Borrowed(data_points));
    let data_serialized = postcard::to_allocvec(&data_message);
    data_serialized
}

pub fn generate_data_point(data: f64, timestamp: f64, iteration: u64, measurement: adc::Measurement) -> DataPoint {
    let data_point = DataPoint {
        value: data,
        timestamp: timestamp,
        channel: iteration_to_node_id(measurement, iteration).unwrap(),
        channel_type: measurement_to_channel_type(iteration_to_node_id(measurement, iteration).unwrap(), measurement).unwrap(),
    };

    return data_point;
}

fn iteration_to_node_id(measurement: adc::Measurement, iteration: u64) -> Option<u32> {
    match measurement {
        adc::Measurement::CurrentLoopPt | adc::Measurement::IValve | adc::Measurement::VValve => {
            return u32::try_from((iteration % 6) + 1).ok();
            // return u32::try_from((iteration % 2) + 1).ok();
        }
        adc::Measurement::VPower => {
            return u32::try_from((iteration % 5) + 1).ok();
        }
        adc::Measurement::IPower | adc::Measurement::Rtd => {
            return u32::try_from((iteration % 2) + 1).ok();
        }
        adc::Measurement::DiffSensors | adc::Measurement::Tc1 | adc::Measurement::Tc2 => {
            return u32::try_from((iteration % 3) + 1).ok();
        }
    }
}

fn measurement_to_channel_type(node_id: u32, measurement: adc::Measurement) -> Option<common::comm::ChannelType> {
    match (node_id, measurement) {
        (_, adc::Measurement::CurrentLoopPt) => Some(common::comm::ChannelType::CurrentLoop),
        (_, adc::Measurement::VValve) => Some(common::comm::ChannelType::ValveVoltage),
        (_, adc::Measurement::IValve) => Some(common::comm::ChannelType::ValveCurrent),
        // (0, adc::Measurement::VPower) => Some(common::comm::ChannelType::RailVoltage),
        // (1, adc::Measurement::VPower) => Some(common::comm::ChannelType::RailVoltage),
        // (2, adc::Measurement::VPower) => Some(common::comm::ChannelType::RailVoltage), // Digital
        // (3, adc::Measurement::VPower) => Some(common::comm::ChannelType::RailVoltage), // Analog 
        // (4, adc::Measurement::VPower) => Some(common::comm::ChannelType::RailVoltage),
        // (0, adc::Measurement::IPower) => Some(common::comm::ChannelType::RailCurrent), // 24V
        // (1, adc::Measurement::IPower) => Some(common::comm::ChannelType::RailCurrent), // 5V
        (_, adc::Measurement::VPower) => Some(common::comm::ChannelType::RailVoltage),
        (_, adc::Measurement::IPower) => Some(common::comm::ChannelType::RailCurrent), // 24V
        (_, adc::Measurement::DiffSensors) => Some(common::comm::ChannelType::DifferentialSignal),
        (_, adc::Measurement::Rtd) => Some(common::comm::ChannelType::Rtd),
        (_, adc::Measurement::Tc1) => Some(common::comm::ChannelType::Tc),
        (_, adc::Measurement::Tc2) => Some(common::comm::ChannelType::Tc),
        (_, _) => None,
    }
}
