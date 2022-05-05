use std::collections::HashMap;

use fitparser::{FitDataRecord, Value};
use log::warn;
use serde::Serialize;

use crate::FitDataRecordExt;

type CreatorDevice = FitDevice;
type AntDeviceNumber = u16;

#[derive(Default, Debug, Serialize)]
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
) -> (CreatorDevice, HashMap<AntDeviceNumber, FitDevice>) {
    let mut creator = FitDevice::default();
    let mut devices = HashMap::new();

    iter.filter(|m| m.kind() == fitparser::profile::MesgNum::DeviceInfo)
        .for_each(|msg| {
            let device_index = msg.data_field("device_index").unwrap().value();

            if device_index == &Value::String("creator".into()) {
                creator.extend(&msg);
            } else {
                let field_value = msg.data_field("ant_device_number").map(|f| f.value());
                if let Some(Value::UInt16z(ant_device_number)) = field_value {
                    let device = FitDevice::new(&msg);
                    devices
                        .entry(*ant_device_number)
                        .and_modify(|d: &mut FitDevice| d.extend(&msg))
                        .or_insert(device);
                } else {
                    warn!("DeviceInfo without ant_device_number: {msg:?}")
                }
            }
        });

    (creator, devices)
}

#[cfg(test)]
mod test {
    use crate::streaming_fit_decoder::StreamingFitDecoder;

    use super::extract_devices;

    #[test]
    fn test_extraction() {
        let msgs = StreamingFitDecoder::new(crate::test_fixtures::DATA_INFLATED)
            .into_iterator()
            .map(|r| r.unwrap());
        let (creator, devices) = extract_devices(msgs);

        println!("{}", serde_json::to_string(&creator).unwrap());
        println!("{}", serde_json::to_string(&devices).unwrap());
    }
}
