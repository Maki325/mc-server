use crate::{connection::State, error::Error, io::ReadMCExt, packet::Packet, result::Result};
use async_trait::async_trait;
use byteorder::BigEndian;
use std::marker::{Send, Unpin};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub struct HandshakePacketToServer {
  pub protocol_verision: u64,
  pub server_address: String,
  pub server_port: u16,
  pub next_state: State,
}

async fn read_next_state<R>(buf: &mut R) -> Result<State>
where
  R: AsyncReadExt + Unpin + Send,
{
  let value = buf.read_var_int(true).await?;

  return match value {
    1 => Ok(State::Status),
    2 => Ok(State::Login),
    _ => Err(Error::UnknownNextState(value)),
  };
}

impl HandshakePacketToServer {
  pub async fn new<R>(buf: &mut R) -> Result<HandshakePacketToServer>
  where
    R: AsyncReadExt + Unpin + Send,
  {
    return Ok(HandshakePacketToServer {
      protocol_verision: buf.read_var_int(true).await?,
      server_address: buf.read_string().await?,
      server_port: ReadMCExt::read_u16::<BigEndian>(buf).await?,
      next_state: read_next_state(buf).await?,
    });
  }
}

#[async_trait]
impl Packet for HandshakePacketToServer {
  type Output = HandshakePacketToServer;

  async fn deserialize<R>(buf: &mut R) -> Result<HandshakePacketToServer>
  where
    R: AsyncReadExt + Unpin + Send,
  {
    let _size = buf.get_packet_size().await?;
    let id = buf.read_var_int(true).await?;
    let packet = match id {
      0 => Ok(HandshakePacketToServer::new(buf).await?),
      _ => Err(Error::UnknownPacket(State::Handshake, id)),
    };

    return packet;
  }

  fn get_id(&self) -> u64 {
    0
  }

  fn size_of(&self) -> Result<usize> {
    unreachable!("ToServer packets should not call size_of!");
  }

  async fn serialize<W>(&self, _buf: &mut W) -> Result<usize>
  where
    W: AsyncWriteExt + Unpin + Send,
  {
    unreachable!("ToServer packets should not be serialized!");
  }
}
