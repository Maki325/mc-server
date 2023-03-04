pub mod to_client;
pub mod to_server;

use std::io::{self, ErrorKind, Read, Write};

use crate::{error::Error, read::ReadMCExt, result::Result};

pub trait Packet {
  type Output;

  fn deserialize(buf: &mut impl Read) -> Result<Self::Output>;
  fn serialize(&self, buf: &mut impl Write) -> Result<usize>;
  fn get_id(&self) -> u64;
  fn size_of(&self) -> Result<usize>;
}

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

pub fn get_packet_size(buf: &mut impl Read) -> Result<u64> {
  return match read_var_int_(buf, false) {
    Err(e) => {
      if let Error::IO(e) = &e {
        if std::io::Error::kind(e) == ErrorKind::UnexpectedEof {
          return Err(Error::NoPacketToReceive);
        }
      }
      return Err(e);
    }
    Ok(value) => Ok(value),
  };
}

pub fn read_var_int(buf: &mut impl Read) -> Result<u64> {
  return read_var_int_(buf, true);
}

pub fn read_var_int_(buf: &mut impl Read, blocking: bool) -> Result<u64> {
  let mut value: u64 = 0;
  let mut position = 0;

  loop {
    let current_byte = {
      if blocking {
        buf.read_u8()
      } else {
        byteorder::ReadBytesExt::read_u8(buf)
      }
    }?;
    value |= ((current_byte & SEGMENT_BITS) << position) as u64;

    if (current_byte & CONTINUE_BIT) == 0 {
      break;
    }

    position += 7;

    if position >= 32 {
      return Err(Error::VarIntTooBig);
    }
  }

  return Ok(value);
}

pub fn read_string(buf: &mut impl Read) -> Result<String> {
  let len = read_var_int(buf)?;

  let mut data: Vec<u8> = vec![0; len as usize];
  buf.read_exact_blocking(&mut data[..])?;

  return String::from_utf8(data).map_err(|e| io::Error::new(ErrorKind::Other, e).into());
}
