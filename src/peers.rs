use std::net::SocketAddr;

use hex::ToHex;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

use crate::{error::TorrentError, torrent_file::MetaInfoFile, PEER_ID};

#[derive(Debug)]
pub struct Peer {
    stream: TcpStream,
    peer_id: [u8; 20],
}

impl Peer {
    pub async fn connect(
        addr: &SocketAddr,
        meta_info: &MetaInfoFile,
    ) -> Result<Self, TorrentError> {
        // TCP connect
        let mut stream = TcpStream::connect(addr).await?;

        // Send handshake
        stream.write_u8(19).await?;
        stream.write_all(b"BitTorrent protocol").await?;
        stream.write_all(&[0, 0, 0, 0, 0, 0, 0, 0]).await?;
        stream.write_all(&meta_info.info.info_hash_bytes()).await?;
        stream.write_all(PEER_ID.as_bytes()).await?;
        stream.flush().await?;

        // Read handshake response header
        let mut response_payload = [0; 48];
        stream.read_exact(&mut response_payload).await?;

        // Read handshake response peer ID
        let mut peer_id = [0; 20];
        stream.read_exact(&mut peer_id).await?;

        Ok(Self { stream, peer_id })
    }

    pub fn id(&self) -> String {
        self.peer_id.encode_hex()
    }

    pub async fn read_message(&mut self) -> Result<PeerMessage, TorrentError> {
        PeerMessage::read(&mut self.stream).await
    }

    pub async fn send_message(&mut self, msg: &PeerMessage) -> Result<(), TorrentError> {
        msg.write(&mut self.stream).await
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PeerMessage {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    BitField(Vec<u8>),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
}

impl PeerMessage {
    const MSG_ID_CHOKE: u8 = 0;
    const MSG_ID_UNCHOKE: u8 = 1;
    const MSG_ID_INTERESTED: u8 = 2;
    const MSG_ID_NOT_INTERESTED: u8 = 3;
    const MSG_ID_HAVE: u8 = 4;
    const MSG_ID_BIT_FIELD: u8 = 5;
    const MSG_ID_REQUEST: u8 = 6;
    const MSG_ID_PIECE: u8 = 7;
    const MSG_ID_CANCEL: u8 = 8;

    pub async fn read<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self, TorrentError> {
        let msg_size = reader.read_u32().await?;
        let msg_id_val = reader.read_u8().await?;

        match msg_id_val {
            Self::MSG_ID_CHOKE => Ok(PeerMessage::Choke),
            Self::MSG_ID_UNCHOKE => Ok(PeerMessage::Unchoke),
            Self::MSG_ID_INTERESTED => Ok(PeerMessage::Interested),
            Self::MSG_ID_NOT_INTERESTED => Ok(PeerMessage::NotInterested),
            Self::MSG_ID_HAVE => {
                assert_eq!(msg_size, 5, "Invalid have message size");
                let piece_id = reader.read_u32().await?;
                Ok(PeerMessage::Have(piece_id))
            }
            Self::MSG_ID_BIT_FIELD => {
                let mut block = vec![0; (msg_size - 1) as usize];
                reader.read_exact(&mut block).await?;
                Ok(PeerMessage::BitField(block))
            }
            Self::MSG_ID_REQUEST => {
                assert_eq!(msg_size, 13, "Invalid request message size");
                let index = reader.read_u32().await?;
                let begin = reader.read_u32().await?;
                let length = reader.read_u32().await?;
                Ok(PeerMessage::Request {
                    index,
                    begin,
                    length,
                })
            }
            Self::MSG_ID_PIECE => {
                let index = reader.read_u32().await?;
                let begin = reader.read_u32().await?;
                let mut block = vec![0; (msg_size - 9) as usize];
                reader.read_exact(&mut block).await?;
                Ok(PeerMessage::Piece {
                    index,
                    begin,
                    block,
                })
            }
            Self::MSG_ID_CANCEL => {
                assert_eq!(msg_size, 13, "Invalid cancel message size");
                let index = reader.read_u32().await?;
                let begin = reader.read_u32().await?;
                let length = reader.read_u32().await?;
                Ok(PeerMessage::Cancel {
                    index,
                    begin,
                    length,
                })
            }
            _ => Err(TorrentError::InvalidMessageId),
        }
    }

    pub async fn write<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<(), TorrentError> {
        match self {
            PeerMessage::Choke => {
                writer.write_u32(1).await?;
                writer.write_u8(0).await?;
            }
            PeerMessage::Unchoke => {
                writer.write_u32(1).await?;
                writer.write_u8(1).await?;
            }
            PeerMessage::Interested => {
                writer.write_u32(1).await?;
                writer.write_u8(2).await?;
            }
            PeerMessage::NotInterested => {
                writer.write_u32(1).await?;
                writer.write_u8(3).await?;
            }
            PeerMessage::Have(piece_id) => {
                writer.write_u32(5).await?;
                writer.write_u8(4).await?;
                writer.write_u32(*piece_id).await?;
            }
            PeerMessage::BitField(block) => {
                writer.write_u32(block.len() as u32 + 1).await?;
                writer.write_u8(5).await?;
                writer.write_all(block).await?;
            }
            PeerMessage::Request {
                index,
                begin,
                length,
            } => {
                writer.write_u32(13).await?;
                writer.write_u8(6).await?;
                writer.write_u32(*index).await?;
                writer.write_u32(*begin).await?;
                writer.write_u32(*length).await?;
            }
            PeerMessage::Piece {
                index,
                begin,
                block,
            } => {
                writer.write_u32(block.len() as u32 + 9).await?;
                writer.write_u8(7).await?;
                writer.write_u32(*index).await?;
                writer.write_u32(*begin).await?;
                writer.write_all(block).await?;
            }
            PeerMessage::Cancel {
                index,
                begin,
                length,
            } => {
                writer.write_u32(13).await?;
                writer.write_u8(8).await?;
                writer.write_u32(*index).await?;
                writer.write_u32(*begin).await?;
                writer.write_u32(*length).await?;
            }
        };
        Ok(())
    }
}
