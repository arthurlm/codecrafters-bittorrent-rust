use std::{fs, path::PathBuf};

use bittorrent_starter_rust::{bencode_format::BencodeValue, torrent_file::MetaInfoFile, trackers};
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
    Peers { path: PathBuf },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Decode { encoded_text } => {
            let (_, decoded_value) =
                BencodeValue::parse(encoded_text.as_bytes()).expect("Invalid bencode value");
            let decoded_json: serde_json::Value = decoded_value.into();

            println!("{decoded_json}");
        }
        Commands::Info { path } => {
            let meta_info = read_file(path);

            println!("Tracker URL: {}", meta_info.announce);
            println!("Length: {}", meta_info.info.length);
            println!("Info Hash: {}", meta_info.info.info_hash());
            println!("Piece Length: {}", meta_info.info.piece_length);
            println!("Piece Hashes:");
            for piece_hash in meta_info.info.pieces_hashes() {
                println!("{piece_hash}");
            }
        }
        Commands::Peers { path } => {
            let meta_info = read_file(path);
            let response = trackers::query(meta_info).await.unwrap();
            for peer_addr in response.peer_addrs() {
                println!("{peer_addr}");
            }
        }
    }
}

fn read_file(path: PathBuf) -> MetaInfoFile {
    let encoded_data = fs::read(path).expect("Fail to read file");
    let (_, decoded_value) = BencodeValue::parse(&encoded_data).expect("Invalid bencode value");
    serde_json::from_value(decoded_value.into()).expect("Fail to decode torrent file")
}
