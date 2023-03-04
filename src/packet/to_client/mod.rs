use self::status::StatusPacketToClient;
use super::Packet;
use crate::result::Result;
use std::{fmt::Display, io::Write};

pub mod status;

#[derive(Debug)]
pub enum PacketToClient {
  Status(StatusPacketToClient),
}

impl PacketToClient {
  pub fn serialize(&self, buf: &mut impl Write) -> Result<usize> {
    match self {
      PacketToClient::Status(packet) => packet.serialize(buf),
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
