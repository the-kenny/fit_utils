use std::collections::HashMap;

use fitparser::{FitDataRecord, Value};
use log::warn;
use serde::Serialize;

use crate::FitDataRecordExt;

type CreatorDevice = FitDevice;

#[derive(Default, Debug, Serialize, PartialEq)]
pub struct FitDevice {
    #[serde(flatten)]
    fields: HashMap<String, fitparser::Value>,
}

impl FitDevice {
    pub fn new(msg: &FitDataRecord) -> Self {
        let fields = msg
            .fields()
            .iter()
            .map(|f| (f.name().to_string(), f.value().clone()))
            .collect();

        Self { fields }
    }

    pub fn timestamp(&self) -> Option<&chrono::DateTime<chrono::Local>> {
        if let Some(Value::Timestamp(ts)) = self.fields.get("timestamp") {
            Some(ts)
        } else {
            None
        }
    }

    pub fn extend(&mut self, msg: &FitDataRecord) {
        let other = FitDevice::new(msg);

        let ts = self.timestamp();
        let other_ts = other.timestamp();

        let merge = match (ts, other_ts) {
            (Some(ts), Some(other_ts)) if ts < other_ts => true,
            (None, Some(_)) => true,
            (None, None) => true,
            _ => false,
        };

        if merge {
            self.fields.extend(other.fields)
        }
    }
}

pub fn extract_devices(
    iter: impl Iterator<Item = FitDataRecord>,
) -> (CreatorDevice, Vec<FitDevice>) {
    let mut creator = FitDevice::default();
    let mut devices_by_ant_device_number = HashMap::new();
    let mut devices_by_serial_number = HashMap::new();

    iter.for_each(|msg| match &msg.kind() {
        fitparser::profile::MesgNum::DeviceInfo => {
            let device_index = msg.data_field("device_index").unwrap().value();

            if device_index == &Value::String("creator".into()) {
                creator.extend(&msg);
            } else {
                let ant_device_number = msg.data_field("ant_device_number").map(|f| f.value());
                let serial_number = msg.data_field("serial_number").map(|f| f.value());

                if let Some(Value::UInt16z(ant_device_number)) = ant_device_number {
                    let device = FitDevice::new(&msg);
                    devices_by_ant_device_number
                        .entry(*ant_device_number)
                        .and_modify(|d: &mut FitDevice| d.extend(&msg))
                        .or_insert(device);
                } else if let Some(Value::UInt32z(serial_number)) = serial_number {
                    let device = FitDevice::new(&msg);
                    devices_by_serial_number
                        .entry(*serial_number)
                        .and_modify(|d: &mut FitDevice| d.extend(&msg))
                        .or_insert(device);
                } else {
                    warn!("DeviceInfo without ant_device_number or serial_number: {msg:?}")
                }
            }
        }
        fitparser::profile::MesgNum::Record => {
            if let Some(battery_soc_field) = msg.data_field("battery_soc") {
                creator
                    .fields
                    .insert("battery_soc".into(), battery_soc_field.value().clone());
            }
        }
        _ => (),
    });

    let devices = match (devices_by_ant_device_number, devices_by_serial_number) {
        (dant, dsn) if dant.is_empty() => dsn.into_values().collect(),
        (dant, dsn) if dsn.is_empty() => dant.into_values().collect(),
        (dant, dsn) => {
            warn!("Found devices without ant_device_number and devices without serial_number. Merging (there may be duplicates)");
            dant.into_values().chain(dsn.into_values()).collect()
        }
    };

    (creator, devices)
}

#[cfg(test)]
mod test {
    use crate::{streaming_fit_decoder::StreamingFitDecoder, test_fixtures::TEST_FILES};

    use super::extract_devices;

    #[test]
    fn test_extraction() {
        for &(data, _msg_count) in TEST_FILES {
            let msgs = StreamingFitDecoder::new(data)
                .into_iterator()
                .map(|r| r.unwrap());
            let (_creator, devices) = extract_devices(msgs);

            assert!(devices.len() > 0);

            // println!("{}", serde_json::to_string(&creator).unwrap());
            // println!("{}", serde_json::to_string(&devices).unwrap());
        }
    }

    #[test]
    fn test_device_extraction_2016_garmin_edge520() {
        let data = include_bytes!("test_data/Edge520-PowerTapG3-2016-12-17-11-03-20.fit");
        let msgs = StreamingFitDecoder::new(&data[..])
            .into_iterator()
            .map(|r| r.unwrap());

        let (_creator, devices) = extract_devices(msgs);
        assert_ne!(devices, vec![]);
        assert!(devices.len() > 0);
    }
}
