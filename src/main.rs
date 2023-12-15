use std::{cmp, fs, net::SocketAddr, path::PathBuf};

use bittorrent_starter_rust::{
    bencode_format::BencodeValue,
    error::TorrentError,
    peers::{Peer, PeerMessage},
    torrent_file::MetaInfoFile,
    trackers,
};
use clap::{Parser, Subcommand};
use hex::ToHex;
use sha1::{Digest, Sha1};

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
        piece_id: u32,
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
            output_path,
            meta_info_path,
            piece_id,
            ..
        } => {
            let contents = download_piece(meta_info_path, piece_id)
                .await
                .expect("Fail to download msg piece");
            fs::write(output_path, contents).expect("Fail to write data to disk");
        }
    }
}

fn read_file(path: PathBuf) -> MetaInfoFile {
    let encoded_data = fs::read(path).expect("Fail to read file");
    let (_, decoded_value) = BencodeValue::parse(&encoded_data).expect("Invalid bencode value");
    serde_json::from_value(decoded_value.into()).expect("Fail to decode torrent file")
}

async fn download_piece(meta_info_path: PathBuf, piece_id: u32) -> Result<Vec<u8>, TorrentError> {
    // Query tracker
    let meta_info = read_file(meta_info_path);
    let tracker_response = trackers::query(&meta_info).await?;

    // Take 1st peer
    let peer_addrs = tracker_response.peer_addrs();
    let peer_addr = peer_addrs.first().expect("No peer for given .torrent");

    // Connect to it
    let mut peer = Peer::connect(peer_addr).await?;
    peer.send_handshake(&meta_info).await?;

    // Read first message which should be a bit field
    assert!(matches!(
        peer.read_message().await?,
        PeerMessage::BitField(_)
    ));

    // Send interested message
    peer.send_message(&PeerMessage::Interested).await?;

    // Wait for unchoke
    while let Ok(msg) = peer.read_message().await {
        if msg == PeerMessage::Unchoke {
            break;
        }
        eprintln!("Received unexpected message: {msg:?}");
    }

    // Send request for each piece and wait for its response.
    // TODO: Add pipelining
    const CHUNK_SIZE: u32 = 16 << 10;

    let piece_length = cmp::min(
        meta_info.info.piece_length * (piece_id + 1), // last byte for a given piece ID
        meta_info.info.length,
    )
    .saturating_sub(meta_info.info.piece_length * piece_id);

    let mut begin_byte = 0;
    let mut chunk_count = 0;
    loop {
        let end_byte = cmp::min(begin_byte + CHUNK_SIZE, piece_length);
        if end_byte <= begin_byte {
            break;
        }

        let req = PeerMessage::Request {
            index: piece_id,
            begin: begin_byte,
            length: end_byte - begin_byte,
        };
        peer.send_message(&req).await?;

        begin_byte += CHUNK_SIZE;
        chunk_count += 1;
    }

    // Read response chunk
    let mut chunks = Vec::with_capacity(chunk_count);

    while chunks.len() != chunks.capacity() {
        match peer.read_message().await? {
            PeerMessage::Piece {
                index,
                begin,
                block,
            } => {
                assert_eq!(index, piece_id, "Received bad piece ID");
                chunks.push((begin, block));
            }
            msg => eprintln!("Received unexpected message: {msg:?}"),
        }
    }

    // Reorder chunk and flatten the data
    chunks.sort_by_key(|x| x.0);
    let contents: Vec<_> = chunks.into_iter().map(|x| x.1).flatten().collect();

    // Check signature
    let mut hasher = Sha1::new();
    hasher.update(&contents);
    let contents_hash: [u8; 20] = hasher.finalize().into();
    let expected_hash = &meta_info.info.pieces_hashes()[piece_id as usize];
    assert_eq!(contents_hash.encode_hex::<String>(), *expected_hash);

    Ok(contents)
}
