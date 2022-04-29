use std::{
    fs::File,
    io::{stdin, BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use log::{debug, error};

use fit_utils::{inflate, streaming_fit_decoder::StreamingFitDecoder};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    wgs84: bool,
    #[clap(long)]
    skip_undecodeable: bool,
    fit_files: Vec<PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    debug!("Args: {args:?}");

    if args.fit_files.is_empty() {
        let stdin = stdin();
        let input = stdin.lock();

        process_stream(&args, input)
    } else {
        for file in &args.fit_files {
            let input = BufReader::new(File::open(file)?);
            let input = inflate(input)?;
            process_stream(&args, input)?
        }
        Ok(())
    }
}

fn process_stream<R: Read>(args: &Args, input: R) -> Result<(), anyhow::Error> {
    let fit_iter = StreamingFitDecoder::new(input).into_iterator();

    // Handle errors
    let fit_iter = fit_iter.flat_map(|r| match r {
        Ok(r) => Some(r),
        Err(e) if args.skip_undecodeable => panic!("Failed to decode fit message: {e}"),
        Err(e) => {
            error!("Failed to decode fit message {e}");
            None
        }
    });

    // WGS84 Pipeline
    let fit_iter = fit_iter.map(|mut r| {
        if args.wgs84 {
            fit_utils::normalize_wgs84(&mut r);
            r
        } else {
            r
        }
    });

    for record in fit_iter {
        println!("{}", fit_utils::to_json(&record)?);
    }

    Ok(())
}
