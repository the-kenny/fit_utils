use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use log::debug;

use fit_utils::inflate;

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

        let mut fit = fitparser::from_reader(&mut reader).expect("Fit parsing failed");

        if args.wgs84 {
            for r in &mut fit {
                fit_utils::normalize_wgs84(r);
            }
        }

        for record in &fit {
            println!("{}", fit_utils::to_json(record)?.to_string());
        }
    }

    Ok(())
}
