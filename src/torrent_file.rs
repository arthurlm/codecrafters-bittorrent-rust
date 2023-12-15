use std::collections::BTreeMap;

use hex::ToHex;
use serde::Deserialize;

use crate::{bencode_format::*, utils::hash_sha1};

#[derive(Debug, Deserialize)]
pub struct MetaInfoFile {
    pub announce: String,
    pub info: InfoSingleFile,
    pub created_by: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InfoSingleFile {
    pub name: String,
    pub length: u32,
    #[serde(rename = "piece length")]
    pub piece_length: u32,
    pub pieces: Vec<u8>,
}

impl InfoSingleFile {
    pub fn info_hash_bytes(&self) -> [u8; 20] {
        let content = BencodeValue::Dict(BTreeMap::from([
            (
                BencodeText::new(b"name"),
                BencodeValue::Data(BencodeText::new(self.name.as_bytes())),
            ),
            (
                BencodeText::new(b"length"),
                BencodeValue::Integer(self.length as i64),
            ),
            (
                BencodeText::new(b"piece length"),
                BencodeValue::Integer(self.piece_length as i64),
            ),
            (
                BencodeText::new(b"pieces"),
                BencodeValue::Data(BencodeText::new(&self.pieces)),
            ),
        ]));

        let mut buf = Vec::with_capacity(512);
        content
            .encode(&mut buf)
            .expect("Fail to bencode info in memory");

        hash_sha1(&buf)
    }

    pub fn info_hash(&self) -> String {
        self.info_hash_bytes().encode_hex()
    }

    pub fn pieces_count(&self) -> usize {
        assert_eq!(self.pieces.len() % 20, 0, "pieces is not a multiple of 20");
        self.pieces.len() / 20
    }

    pub fn pieces_hashes(&self) -> Vec<String> {
        assert_eq!(self.pieces.len() % 20, 0, "pieces is not a multiple of 20");
        self.pieces
            .chunks(20)
            .map(|piece| piece.encode_hex())
            .collect()
    }
}
