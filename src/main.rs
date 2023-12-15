use std::{fs, net::SocketAddr, path::PathBuf};

use bittorrent_starter_rust::{
    bencode_format::BencodeValue, peers::Peer, torrent_file::MetaInfoFile, trackers,
};
use clap::{Parser, Subcommand};
use hex::ToHex;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
#[non_exhaustive]
enum Commands {
    Decode {
        encoded_text: String,
    },
    Info {
        path: PathBuf,
    },
    Peers {
        path: PathBuf,
    },
    Handshake {
        path: PathBuf,
        addr: SocketAddr,
    },
    #[clap(alias = "download_piece")]
    DownloadPiece {
        #[arg(short = 'o')]
        output_path: PathBuf,
        meta_info_path: PathBuf,
        piece_id: usize,
    },
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
            let tracker_response = trackers::query(&meta_info)
                .await
                .expect("Fail to query tracker");

            for peer_addr in tracker_response.peer_addrs() {
                println!("{peer_addr}");
            }
        }
        Commands::Handshake { path, addr } => {
            let meta_info = read_file(path);

            let mut peer = Peer::connect(&addr).await.expect("Fail to connect peer");
            let peer_id = peer
                .send_handshake(&meta_info)
                .await
                .expect("Fail to send handshake");

            println!("Peer ID: {}", peer_id.encode_hex::<String>());
        }
        Commands::DownloadPiece {
            // output_path,
            meta_info_path,
            // piece_id,
            ..
        } => {
            let meta_info = read_file(meta_info_path);
            let tracker_response = trackers::query(&meta_info)
                .await
                .expect("Fail to query tracker");

            let peer_addrs = tracker_response.peer_addrs();
            let peer_addr = peer_addrs.first().expect("No peer for given .torrent");

            let mut peer = Peer::connect(peer_addr)
                .await
                .expect("Fail to connect peer");

            peer.send_handshake(&meta_info)
                .await
                .expect("Fail to send handshake");

            println!("Waiting for message from {peer:?}");
            while let Ok(msg) = peer.read_message().await {
                println!("{msg:?}");
            }
        }
    }
}

fn read_file(path: PathBuf) -> MetaInfoFile {
    let encoded_data = fs::read(path).expect("Fail to read file");
    let (_, decoded_value) = BencodeValue::parse(&encoded_data).expect("Invalid bencode value");
    serde_json::from_value(decoded_value.into()).expect("Fail to decode torrent file")
}
