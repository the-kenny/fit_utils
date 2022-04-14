use std::io::{Read, Seek};

use fitparser::{FitDataField, FitDataRecord};
use geo_types::Coordinate;

pub trait FitDataRecordExt {
    fn data_field(&self, field_name: &str) -> Option<&FitDataField>;
    fn coordinates_wgs84(&self) -> Option<geo_types::Coordinate<f32>>;
}

impl FitDataRecordExt for FitDataRecord {
    fn data_field(&self, field_name: &str) -> Option<&FitDataField> {
        self.fields().iter().find(|f| f.name() == field_name)
    }

    fn coordinates_wgs84(&self) -> Option<Coordinate<f32>> {
        let lat = self.data_field("position_lat").map(|x| x.value());
        let lon = self.data_field("position_long").map(|x| x.value());

        use fitparser::Value::SInt32;
        if let (Some(&SInt32(lat)), Some(&SInt32(lon))) = (lat, lon) {
            let lat = (lat as f32) * 180.0 / (2.0 as f32).powf(31.0);
            let lon = (lon as f32) * 180.0 / (2.0 as f32).powf(31.0);
            Some(Coordinate { x: lon, y: lat })
        } else {
            None
        }
    }
}

pub fn semicircles_to_wgs84(ss: i32) -> f32 {
    (ss as f32) * 180.0 / (2.0 as f32).powf(31.0)
}

pub fn inflate<'a, In: Read + Seek + 'a>(reader: In) -> Result<Box<dyn Read + 'a>, std::io::Error> {
    let gz = flate2::read::GzDecoder::new(reader);
    if gz.header().is_none() {
        let mut seekable = gz.into_inner();
        seekable.seek(std::io::SeekFrom::Start(0))?;
        Ok(Box::new(seekable))
    } else {
        Ok(Box::new(gz))
    }
}

pub fn normalize_wgs84(record: &mut FitDataRecord) {
    for field_name in ["position_lat", "position_long"] {
        if let Some(field) = record.data_field(field_name) {
            if let &fitparser::Value::SInt32(ss) = field.value() {
                let wgs84 = semicircles_to_wgs84(ss);
                let field = fitparser::FitDataField::new(
                    format!("{field_name}_wgs84"),
                    field.number(),
                    fitparser::Value::Float32(wgs84),
                    "wgs84".into(),
                );
                record.push(field);
            }
        }
    }
}

pub fn to_json(record: &FitDataRecord) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(record)
}
