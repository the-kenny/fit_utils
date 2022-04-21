use fitparser::{
    de::{FitObject, FitStreamProcessor},
    FitDataRecord,
};
use log::debug;

pub struct FitDecoder {
    processor: FitStreamProcessor,
    buffer: Vec<u8>,
}

#[derive(Debug)]
pub enum FitDecodeResult {
    NotEnoughData,
    Record(FitDataRecord),
}

#[derive(Debug, thiserror::Error)]
pub enum FitDecoderError {
    #[error("fitparser error: {0}")]
    Fitparser(String),
}

impl From<fitparser::ErrorKind> for FitDecoderError {
    fn from(k: fitparser::ErrorKind) -> Self {
        Self::Fitparser(k.to_string())
    }
}

impl From<Box<fitparser::ErrorKind>> for FitDecoderError {
    fn from(k: Box<fitparser::ErrorKind>) -> Self {
        Self::Fitparser(k.to_string())
    }
}

impl FitDecoder {
    pub fn new() -> Self {
        let processor = FitStreamProcessor::new();
        let buffer = vec![];
        FitDecoder { processor, buffer }
    }

    pub fn poll(&mut self) -> Result<FitDecodeResult, FitDecoderError> {
        let len = self.buffer.len();
        debug!("self.buffer len={len}");
        loop {
            match self.processor.deserialize_next(&self.buffer) {
                Ok((rest, object)) => {
                    // Hack: Strip self.buffer.len() - rest.len() from self.buffer to avoid copying
                    let to_drain = self.buffer.len() - rest.len();
                    self.buffer.drain(0..to_drain);

                    // self.buffer = Vec::from(rest);
                    if let FitObject::DataMessage(msg) = object {
                        let msg = self.processor.decode_message(msg)?;
                        return Ok(FitDecodeResult::Record(msg));
                    }
                }
                Err(e) if matches!(*e, fitparser::ErrorKind::UnexpectedEof(_)) => {
                    return Ok(FitDecodeResult::NotEnoughData)
                }
                Err(e) => return Err(FitDecoderError::from(e)),
            }
        }
    }

    pub fn add_chunk(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
    }

    // fn process_obj(&mut self, obj: FitObject) -> Result<(), Box<dyn std::error::Error>> {
    //     match obj {
    //         FitObject::Crc(v) => {
    //             println!("CRC Value: {}", v)
    //         }
    //         FitObject::Header(v) => {
    //             self.processor.reset();
    //             println!(
    //                 "New FIT file, protocol version: {:?}, profile version: {:?}, data length: {}",
    //                 v.protocol_ver_enc(),
    //                 v.profile_ver_enc(),
    //                 v.data_size()
    //             );
    //         }
    //         FitObject::DataMessage(msg) => {
    //             let record = self.processor.decode_message(msg)?;
    //             let kind = record.kind();
    //             let mut fields: Vec<String> = record
    //                 .into_vec()
    //                 .into_iter()
    //                 .map(|f| {
    //                     format!(
    //                         "{}: {}",
    //                         f.name().to_owned(),
    //                         fitparser::ValueWithUnits::from(f)
    //                     )
    //                 })
    //                 .collect();
    //             fields.sort();
    //             println!(
    //                 "{} data message (global message number {}): => {}",
    //                 kind,
    //                 kind.as_u16(),
    //                 fields.join(", ")
    //             );
    //         }
    //         FitObject::DefinitionMessage(msg) => {
    //             println!(
    //                 "definition message {}: {} message (global message number {}) with {} fields",
    //                 msg.local_message_number(),
    //                 MesgNum::from(msg.global_message_number()),
    //                 msg.global_message_number(),
    //                 msg.field_definitions().len() + msg.developer_field_definitions().len()
    //             );
    //         }
    //     }

    //     Ok(())
    // }
}
