use std::{io, net::SocketAddr};

use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

use crate::{error::TorrentError, torrent_file::MetaInfoFile, PEER_ID};

#[derive(Debug)]
pub struct Peer {
    stream: TcpStream,
}

impl Peer {
    pub async fn connect(addr: &SocketAddr) -> Result<Self, TorrentError> {
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

    pub async fn read_message(&mut self) -> Result<PeerMessage, TorrentError> {
        PeerMessage::read(&mut self.stream).await
    }

    pub async fn send_message(&mut self, msg: &PeerMessage) -> Result<(), TorrentError> {
        msg.write(&mut self.stream).await
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Invalid message ID")]
pub struct InvalidMessageId;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PeerMessageId {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    BitField,
    Request,
    Piece,
    Cancel,
}

impl TryFrom<u8> for PeerMessageId {
    type Error = InvalidMessageId;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Choke),
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => Ok(Self::Have),
            5 => Ok(Self::BitField),
            6 => Ok(Self::Request),
            7 => Ok(Self::Piece),
            8 => Ok(Self::Cancel),
            _ => Err(InvalidMessageId),
        }
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
    pub async fn read<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self, TorrentError> {
        let msg_size = reader.read_u32().await?;
        let msg_id_val = reader.read_u8().await?;

        match PeerMessageId::try_from(msg_id_val)? {
            PeerMessageId::Choke => Ok(PeerMessage::Choke),
            PeerMessageId::Unchoke => Ok(PeerMessage::Unchoke),
            PeerMessageId::Interested => Ok(PeerMessage::Interested),
            PeerMessageId::NotInterested => Ok(PeerMessage::NotInterested),
            PeerMessageId::Have => {
                assert_eq!(msg_size, 5, "Invalid have message size");
                let piece_id = reader.read_u32().await?;
                Ok(PeerMessage::Have(piece_id))
            }
            PeerMessageId::BitField => {
                let mut block = vec![0; (msg_size - 1) as usize];
                reader.read_exact(&mut block).await?;
                Ok(PeerMessage::BitField(block))
            }
            PeerMessageId::Request => {
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
            PeerMessageId::Piece => {
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
            PeerMessageId::Cancel => {
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
                writer.write_all(&block).await?;
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
                writer.write_all(&block).await?;
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
