use std::fmt::Display;

use self::{handshake::HandshakePacketToServer, status::StatusPacketToServer};
pub mod handshake;
pub mod status;

#[derive(Debug)]
pub enum PacketToServer {
  Handshake(HandshakePacketToServer),
  Status(StatusPacketToServer),
  // Ping,
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
