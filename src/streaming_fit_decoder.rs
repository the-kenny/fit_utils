use std::io::Read;

use fitparser::FitDataRecord;
use log::debug;
use thiserror::Error;

use crate::fit_decoder::{self, FitDecodeResult, FitDecoder};

pub struct StreamingFitDecoder<R: Read> {
    decoder: FitDecoder,
    reader: R,
    chunk_size: usize,
}

impl<R: Read> StreamingFitDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self::new_with_chunk_size(reader, 512)
    }

    pub fn new_with_chunk_size(reader: R, chunk_size: usize) -> Self {
        Self {
            decoder: FitDecoder::new(),
            reader,
            chunk_size,
        }
    }

    pub fn poll(&mut self) -> Result<Option<FitDataRecord>, StreamingFitDecoderError> {
        loop {
            if let FitDecodeResult::Record(record) = self.decoder.poll()? {
                return Ok(Some(record));
            } else {
                let nread = self.pull_data()?;
                if nread == 0 {
                    return Ok(None);
                }
            }
        }
    }

    fn pull_data(&mut self) -> std::io::Result<usize> {
        let mut chunk = vec![0u8; self.chunk_size];
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

    use crate::test_fixtures::TEST_FILES;

    use super::*;

    use test_case::test_case;

    #[test_case(1; "chunk_size of 1")]
    #[test_case(2; "chunk_size of 2")]
    #[test_case(8; "chunk_size of 8")]
    #[test_case(15; "chunk_size of 15")]
    #[test_case(128; "chunk_size of 128")]
    #[test_case(1024*1024*16; "chunk_size of 11024*1024*16")]
    fn test_streaming(chunk_size: usize) {
        for &(data, msg_count) in TEST_FILES {
            let reader = Cursor::new(data);
            let mut decoder = StreamingFitDecoder::new_with_chunk_size(reader, chunk_size);

            let mut n = 0;
            while let Ok(Some(_)) = decoder.poll() {
                n += 1
            }

            assert_eq!(n, msg_count)
        }
    }

    #[test]
    fn test_iterator() {
        for &(data, msg_count) in TEST_FILES {
            let reader = Cursor::new(data);
            assert_eq!(
                msg_count,
                StreamingFitDecoder::new(reader).into_iterator().count()
            );
        }
    }
}
