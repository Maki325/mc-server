use crate::{
  connection::State,
  error::Error,
  packet::{get_packet_size, read_string, read_var_int, Packet},
  read::ReadMCExt,
  result::Result,
};
use byteorder::BigEndian;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct HandshakePacketToServer {
  pub protocol_verision: u64,
  pub server_address: String,
  pub server_port: u16,
  pub next_state: State,
}

fn read_next_state(buf: &mut impl Read) -> Result<State> {
  let value = read_var_int(buf)?;

  return match value {
    1 => Ok(State::Status),
    2 => Ok(State::Login),
    _ => Err(Error::UnknownNextState(value)),
  };
}

impl HandshakePacketToServer {
  pub fn new(buf: &mut impl Read) -> Result<HandshakePacketToServer> {
    return Ok(HandshakePacketToServer {
      protocol_verision: read_var_int(buf)?,
      server_address: read_string(buf)?,
      server_port: buf.read_u16::<BigEndian>()?,
      next_state: read_next_state(buf)?,
    });
  }
}

impl Packet for HandshakePacketToServer {
  type Output = HandshakePacketToServer;

  fn deserialize(buf: &mut impl Read) -> Result<HandshakePacketToServer> {
    let _size = get_packet_size(buf)?;
    let id = read_var_int(buf)?;
    return match id {
      0 => Ok(HandshakePacketToServer::new(buf)?),
      _ => Err(Error::UnknownPacket(State::Handshake, id)),
    };
  }

  fn get_id(&self) -> u64 {
    0
  }

  fn size_of(&self) -> Result<usize> {
    unreachable!("ToServer packets should not call size_of!");
  }

  fn serialize(&self, _buf: &mut impl Write) -> Result<usize> {
    unreachable!("ToServer packets should not be serialized!");
  }
}
