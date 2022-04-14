use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    path::PathBuf,
};

use clap::Parser;
use log::debug;

use fit_utils::{inflate, semicircles_to_wgs84, FitDataRecordExt};

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

        let mut fit = fitparser::from_reader(&mut reader).unwrap();

        if args.wgs84 {
            for r in &mut fit {
                for field_name in ["position_lat", "position_long"] {
                    if let Some(field) = r.data_field(field_name) {
                        if let &fitparser::Value::SInt32(ss) = field.value() {
                            let wgs84 = semicircles_to_wgs84(ss);
                            let field = fitparser::FitDataField::new(
                                format!("{field_name}_wgs84"),
                                field.number(),
                                fitparser::Value::Float32(wgs84),
                                "wgs84".into(),
                            );
                            r.push(field);
                        }
                    }
                }
            }
        }

        for f in fit {
            println!("{}", serde_json::to_value(f).unwrap().to_string());
        }
    }

    Ok(())
}
