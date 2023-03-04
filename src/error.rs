use std::{fmt::Display, io};

use crate::{connection::State, packet::to_server::PacketToServer};

#[derive(Debug)]
pub enum Error {
  IO(io::Error),
  NoPacketToReceive,
  VarIntTooBig,
  ConnectionAborted(&'static str),
  TimedOut,
  UnexpectedPacket(PacketToServer, &'static str),
  UnknownPacket(State, u64),
  UnknownNextState(u64),
}

impl std::error::Error for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::IO(error) => error.fmt(f),
      Error::VarIntTooBig => write!(f, "VarInt is too big!"),
      Error::ConnectionAborted(reason) => write!(f, "{} {}", "Connection Aborted!", reason),
      Error::TimedOut => write!(f, "TimedOut!"),
      Error::UnexpectedPacket(got, expected) => {
        write!(f, "Unexpected Packet! Expected {} got {}", got, expected)
      }
      Error::NoPacketToReceive => write!(f, "No packet to receive!"),
      Error::UnknownPacket(state, id) => {
        write!(f, "Unknown packet with id {} in state {}!", id, state)
      }
      Error::UnknownNextState(id) => write!(f, "Unknown next state id: {}!", id),
    }
  }
}

impl From<io::Error> for Error {
  fn from(value: io::Error) -> Self {
    return Error::IO(value);
  }
}
