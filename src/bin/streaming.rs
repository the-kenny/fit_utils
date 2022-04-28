use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use log::debug;

use fit_utils::{inflate, streaming_fit_decoder::StreamingFitDecoder};

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
        let reader = inflate(file)?;
        let decoder = StreamingFitDecoder::new(reader);

        decoder.into_iterator().for_each(|msg| println!("{msg:?}"));
    }

    Ok(())
}
