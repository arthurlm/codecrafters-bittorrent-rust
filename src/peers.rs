use std::{io, net::SocketAddr};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{error::TorrentError, torrent_file::MetaInfoFile, PEER_ID};

#[derive(Debug)]
pub struct Peer {
    stream: TcpStream,
}

impl Peer {
    pub async fn connect(addr: SocketAddr) -> Result<Self, TorrentError> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    pub async fn send_handshake(&mut self, meta_info: &MetaInfoFile) -> io::Result<[u8; 20]> {
        // Send request
        let mut request_payload = Vec::<u8>::with_capacity(68);
        request_payload.push(19);
        request_payload.extend(b"BitTorrent protocol");
        request_payload.extend([0, 0, 0, 0, 0, 0, 0, 0]);
        request_payload.extend(meta_info.info.info_hash_bytes());
        request_payload.extend(PEER_ID.as_bytes());
        self.stream.write_all(&request_payload).await?;

        // Read response
        let mut response_payload = [0; 68];
        self.stream.read_exact(&mut response_payload).await?;

        // NOTE: maybe there is a better way to do this ...
        let mut output = [0; 20];
        output.clone_from_slice(&response_payload[48..68]);
        Ok(output)
    }
}
