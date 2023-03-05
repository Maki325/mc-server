use self::{handshake::HandshakePacketToServer, status::StatusPacketToServer};
use std::fmt::Display;

pub mod handshake;
pub mod status;

#[derive(Debug)]
pub enum PacketToServer {
  Handshake(HandshakePacketToServer),
  Status(StatusPacketToServer),
}

impl Display for PacketToServer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "PacketToServer::{}",
      match self {
        PacketToServer::Handshake(..) => "Handshake",
        PacketToServer::Status(..) => "Status",
      }
    )
  }
}
