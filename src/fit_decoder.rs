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

#[derive(Debug)]
pub enum RawFitDecodeResult {
    NotEnoughData,
    Object(fitparser::de::FitObject),
}

#[derive(Debug, thiserror::Error)]
#[error("FitDecoderError: {0}")]
pub struct FitDecoderError(String);

impl From<fitparser::ErrorKind> for FitDecoderError {
    fn from(k: fitparser::ErrorKind) -> Self {
        Self(k.to_string())
    }
}

impl From<Box<fitparser::ErrorKind>> for FitDecoderError {
    fn from(k: Box<fitparser::ErrorKind>) -> Self {
        Self(k.to_string())
    }
}

impl FitDecoder {
    pub fn new() -> Self {
        let processor = FitStreamProcessor::new();
        let buffer = vec![];
        FitDecoder { processor, buffer }
    }

    pub fn poll_raw(&mut self) -> Result<RawFitDecodeResult, FitDecoderError> {
        let len = self.buffer.len();
        debug!("self.buffer len={len}");
        loop {
            match self.processor.deserialize_next(&self.buffer) {
                Ok((rest, object)) => {
                    // Hack: Strip self.buffer.len() - rest.len() from self.buffer to avoid copying
                    let to_drain = self.buffer.len() - rest.len();
                    self.buffer.drain(0..to_drain);

                    return Ok(RawFitDecodeResult::Object(object));
                }
                Err(e) if matches!(*e, fitparser::ErrorKind::UnexpectedEof(_)) => {
                    return Ok(RawFitDecodeResult::NotEnoughData)
                }
                Err(e) => return Err(FitDecoderError::from(e)),
            }
        }
    }

    pub fn poll(&mut self) -> Result<FitDecodeResult, FitDecoderError> {
        loop {
            match self.poll_raw()? {
                RawFitDecodeResult::Object(FitObject::DataMessage(msg)) => {
                    let record = self.processor.decode_message(msg)?;
                    return Ok(FitDecodeResult::Record(record));
                }
                RawFitDecodeResult::Object(_) => continue,
                RawFitDecodeResult::NotEnoughData => return Ok(FitDecodeResult::NotEnoughData),
            }
        }
    }

    pub fn add_chunk(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &'static [u8] = include_bytes!("test_data/22952.fit");
    const EXPECTED: usize = 22952;

    #[test]
    fn test_22952_fit_size() {
        let expected = fitparser::de::from_bytes(DATA).unwrap().len();
        assert_eq!(EXPECTED, expected);
    }

    #[test]
    fn poll_raw() {
        let mut d = FitDecoder::new();

        assert!(matches!(
            d.poll_raw(),
            Ok(RawFitDecodeResult::NotEnoughData)
        ));

        d.add_chunk(&DATA[..]);

        assert!(matches!(
            d.poll_raw(),
            Ok(RawFitDecodeResult::Object(
                fitparser::de::FitObject::Header(_)
            ))
        ));
    }

    #[test]
    fn test_count() {
        let mut d = FitDecoder::new();
        d.add_chunk(&DATA[..]);

        let mut n = 0;
        while let Ok(FitDecodeResult::Record(_)) = d.poll() {
            n += 1;
        }

        assert_eq!(n, EXPECTED);
    }
}
