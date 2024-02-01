//! Tool to download [GOV.UK bank holidays JSON](https://www.gov.uk/bank-holidays.json).
//! Can be used to refresh the cached file used in this library.
//!
//! ```shell
//! cargo run --example download -- src/data_source/bank-holidays.json
//! ```

use std::fs::File;
use std::io::{BufReader, stderr, Write};
use std::path::Path;

use tracing_subscriber::{EnvFilter, filter::LevelFilter, prelude::*};

use govuk_bank_holidays::SOURCE_URL;
use govuk_bank_holidays::data_source::DataSource;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_writer(stderr))
        .with(EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy())
        .init();

    let mut args = std::env::args();
    let program = args.next()
        .and_then(|program| {
            program.rsplit_once('/')
                .map(|(_path, program)| program.to_owned())
        })
        .unwrap_or("download".to_owned());

    if let Some(path) = args.next().as_deref() {
        if let Err(error) = download_to_path(SOURCE_URL, path).await {
            eprintln!("{error}");
            std::process::exit(2);
        }
    } else {
        eprintln!("Download bank holiday information from GOV.UK data\nUsage:\n  {program} [path]   save to a JSON file");
        std::process::exit(1);
    }
}

async fn download_to_path(url: &str, path: &str) -> Result<(), &'static str> {
    let mut data = reqwest::get(url)
        .await
        .map_err(|error| {
            tracing::error!("{error}");
            "Could not download data"
        })?
        .json::<DataSource>()
        .await
        .map_err(|error| {
            tracing::error!("{error}");
            "Could not deserialise data"
        })?;

    let path = Path::new(path);
    if path.is_file() {
        let file = File::open(path)
            .map_err(|error| {
                tracing::error!("{error}");
                "Could not open existing file"
            })?;
        let file = BufReader::new(file);
        let mut existing_data: DataSource = serde_json::from_reader(file)
            .map_err(|error| {
                tracing::error!("{error}");
                "Could not deserialise existing file"
            })?;
        existing_data.merge(data);
        data = existing_data;
    }
    data.sort();
    data.add_missing_divisions();
    let mut file = File::create(path)
        .map_err(|error| {
            tracing::error!("{error}");
            "Could not create file"
        })?;
    serde_json::to_writer_pretty(&mut file, &data)
        .map_err(|error| {
            tracing::error!("{error}");
            "Could not serialise to file"
        })?;
    file.write(b"\n")
        .map(|_| ())
        .map_err(|error| {
            tracing::error!("{error}");
            "Could not write to file"
        })
}
