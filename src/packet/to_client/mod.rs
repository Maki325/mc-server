use self::status::StatusPacketToClient;
use super::Packet;
use crate::result::Result;
use std::{
  fmt::Display,
  marker::{Send, Unpin},
};
use tokio::io::AsyncWriteExt;

pub mod status;

#[derive(Debug)]
pub enum PacketToClient {
  Status(StatusPacketToClient),
}

impl PacketToClient {
  pub async fn serialize<W>(&self, buf: &mut W) -> Result<usize>
  where
    W: AsyncWriteExt + Unpin + Send,
  {
    match self {
      PacketToClient::Status(packet) => packet.serialize(buf).await,
    }
  }
}

impl Display for PacketToClient {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "PacketToClient::{}",
      match self {
        PacketToClient::Status(..) => "Status",
      }
    )
  }
}
