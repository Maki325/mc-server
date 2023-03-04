use crate::{
  connection::State,
  error::Error,
  packet::{get_packet_size, read_var_int, Packet},
  read::ReadMCExt,
  result::Result,
};
use byteorder::BigEndian;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum StatusPacketToServer {
  Status,
  Ping(u64),
}

impl Packet for StatusPacketToServer {
  type Output = StatusPacketToServer;

  fn deserialize(buf: &mut impl Read) -> Result<StatusPacketToServer> {
    let _size = get_packet_size(buf)?;
    let id = read_var_int(buf)?;
    println!("Status Id: {}", id);
    return match id {
      0 => Ok(StatusPacketToServer::Status),
      1 => Ok(StatusPacketToServer::Ping(buf.read_u64::<BigEndian>()?)),
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

  fn serialize(&self, _buf: &mut impl Write) -> Result<usize> {
    unreachable!("ToServer packets should not be serialized!");
  }
}
