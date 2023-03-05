use crate::{connection::State, error::Error, packet::Packet, read::ReadMCExt, result::Result};
use async_trait::async_trait;
use byteorder::BigEndian;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub enum StatusPacketToServer {
  Status,
  Ping(u64),
}

#[async_trait]
impl Packet for StatusPacketToServer {
  type Output = StatusPacketToServer;

  async fn deserialize<R>(buf: &mut R) -> Result<StatusPacketToServer>
  where
    R: AsyncReadExt + Unpin + Send,
  {
    let _size = buf.get_packet_size().await?;
    let id = buf.read_var_int(true).await?;
    return match id {
      0 => Ok(StatusPacketToServer::Status),
      1 => Ok(StatusPacketToServer::Ping(
        ReadMCExt::read_u64::<BigEndian>(buf).await?,
      )),
      _ => Err(Error::UnknownPacket(State::Status, id)),
    };
  }

  fn get_id(&self) -> u64 {
    match self {
      StatusPacketToServer::Status => 0,
      StatusPacketToServer::Ping(_) => 1,
    }
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
