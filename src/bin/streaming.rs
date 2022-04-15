use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use log::debug;

use fit_utils::{
    decoder::FitDecoder, inflate, normalize_wgs84, semicircles_to_wgs84, FitDataRecordExt,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    wgs84: bool,
    fit_files: Vec<PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    debug!("Args: {args:?}");

    for file in args.fit_files {
        let file = BufReader::new(File::open(file)?);
        let mut reader = inflate(file)?;

        let mut decoder = FitDecoder::new();
        loop {
            let mut buf = [0; 1024];
            let read = reader.read(&mut buf)?;
            decoder.add_chunk(&buf[0..read]);

            while let fit_utils::decoder::FitDecodeResult::Record(msg) = decoder.decode()? {
                println!("{msg:?}");
            }

            if read == 0 {
                break;
            }
        }
    }

    Ok(())
}
