use std::io::Read;

use fitparser::FitDataRecord;
use log::debug;
use thiserror::Error;

use crate::fit_decoder::{self, FitDecodeResult, FitDecoder};

const CHUNK_SIZE: usize = 512;

pub struct StreamingFitDecoder<R: Read> {
    decoder: FitDecoder,
    reader: R,
}

impl<R: Read> StreamingFitDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            decoder: FitDecoder::new(),
            reader,
        }
    }

    pub fn poll(&mut self) -> Result<Option<FitDataRecord>, StreamingFitDecoderError> {
        loop {
            if let FitDecodeResult::Record(record) = self.decoder.poll()? {
                return Ok(Some(record));
            } else {
                let nread = dbg!(self.pull_data())?;
                if nread == 0 {
                    return Ok(None);
                }
            }
        }
    }

    fn pull_data(&mut self) -> std::io::Result<usize> {
        let mut chunk = [0u8; CHUNK_SIZE];
        let nread = self.reader.read(&mut chunk)?;
        debug!("Read {nread} bytes");

        self.decoder.add_chunk(&chunk[0..nread]);
        Ok(nread)
    }

    pub fn into_iterator(self) -> FitDecoderIterator<R> {
        FitDecoderIterator(self)
    }
}

pub struct FitDecoderIterator<R: Read>(StreamingFitDecoder<R>);

impl<R: Read> Iterator for FitDecoderIterator<R> {
    type Item = Result<FitDataRecord, StreamingFitDecoderError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.poll() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[derive(Debug, Error)]
pub enum StreamingFitDecoderError {
    #[error("FitDecoder Error: {0}")]
    FitDecoder(#[from] fit_decoder::FitDecoderError),
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    const DATA_INFLATED: &'static [u8] = include_bytes!("test_data/22952.fit");
    const EXPECTED: usize = 22952;

    #[test]
    fn test_streaming() {
        let reader = Cursor::new(DATA_INFLATED);
        let mut decoder = StreamingFitDecoder::new(reader);

        let mut n = 0;
        while let Ok(Some(_)) = decoder.poll() {
            n += 1
        }

        assert_eq!(n, EXPECTED)
    }

    #[test]
    fn test_iterator() {
        let reader = Cursor::new(DATA_INFLATED);
        assert_eq!(
            EXPECTED,
            StreamingFitDecoder::new(reader).into_iterator().count()
        );
    }
}
