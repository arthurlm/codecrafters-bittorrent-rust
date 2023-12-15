pub mod bencode_format;
pub mod error;
pub mod peers;
pub mod torrent_file;
pub mod trackers;
pub mod url_encode;
pub mod utils;

pub const PEER_ID: &str = "AL-20231215-1.0.0.00";

const _: () = {
    if PEER_ID.as_bytes().len() != 20 {
        panic!("Invalid PEER_ID length");
    }
};
