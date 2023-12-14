use std::{fs, path::PathBuf};

use bittorrent_starter_rust::{bencode_format::BencodeValue, torrent_file::MetaInfoFile};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
#[non_exhaustive]
enum Commands {
    Decode { encoded_text: String },
    Info { path: PathBuf },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Decode { encoded_text } => {
            let (_, decoded_value) =
                BencodeValue::parse(encoded_text.as_bytes()).expect("Invalid bencode value");
            let decoded_json: serde_json::Value = decoded_value.into();

            println!("{decoded_json}");
        }
        Commands::Info { path } => {
            let encoded_data = fs::read(path).expect("Fail to read file");

            let (_, decoded_value) =
                BencodeValue::parse(&encoded_data).expect("Invalid bencode value");

            let decoded_file: MetaInfoFile =
                serde_json::from_value(decoded_value.into()).expect("Fail to decode torrent file");

            println!("Tracker URL: {}", decoded_file.announce);
            println!("Length: {}", decoded_file.info.length);
            println!("Info Hash: {}", decoded_file.info.info_hash());
        }
    }
}
