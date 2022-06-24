use std::{
    fs::File,
    io::{stdin, BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use log::{debug, error};

use fit_utils::{inflate, streaming_fit_decoder::StreamingFitDecoder};
use serde_json::json;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    skip_undecodeable: bool,
    fit_files: Vec<PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

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

    let (creator, devices) = fit_utils::devices::extract_devices(fit_iter);

    let json = json!({
        "creator": creator,
        "devices": devices,
    });

    println!("{json}");

    Ok(())
}
