use std::{fs, path::PathBuf};

use bittorrent_starter_rust::bencode_parser::BencodeValue;
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
            println!("{encoded_data:?}");

            let (_, decoded_value) =
                BencodeValue::parse(&encoded_data).expect("Invalid bencode value");
            let decoded_json: serde_json::Value = decoded_value.into();

            println!("{decoded_json:?}");
        }
    }
}
