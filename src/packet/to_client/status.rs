use crate::{
  packet::Packet,
  result::Result,
  write::{var_int_len, WriteMCExt},
};
use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;
use std::io::{ErrorKind, Read, Write};

type MOTD = String;

#[derive(Debug, Serialize)]
pub struct Version {
  name: String,
  protocol: u64,
}

#[derive(Debug, Serialize)]
pub struct Players {
  max: u64,
  online: u64,
  sample: Vec<()>,
}

#[derive(Debug, Serialize)]
pub struct Description {
  text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusData {
  version: Version,
  players: Players,
  description: Description,

  #[serde(skip_serializing_if = "Option::is_none")]
  favicon: Option<String>,
  enforces_secure_chat: bool,
}

impl StatusData {
  pub fn new(motd: MOTD) -> StatusData {
    StatusData {
      version: Version {
        name: "1.19.3".into(),
        protocol: 761,
        // protocol: 249,
      },
      players: Players {
        max: 10,
        online: 2,
        sample: vec![],
      },
      description: Description { text: motd },
      enforces_secure_chat: false,
      favicon: None,
    }
  }
}

#[derive(Debug)]
pub struct Status {
  status_data: String,
}

impl Status {
  pub fn new(motd: MOTD) -> Result<Status> {
    let status_data = StatusData::new(motd);
    Ok(Status {
      status_data: match serde_json::to_string(&status_data) {
        Err(err) => return Err(std::io::Error::new(ErrorKind::Other, err).into()),
        Ok(json) => json,
      },
    })
  }
}

#[derive(Debug)]
pub enum StatusPacketToClient {
  Status(Status),
  Ping(u64),
}

fn serialize_pre(buf: &mut impl Write, packet: &StatusPacketToClient) -> Result<usize> {
  let size = packet.size_of()?;
  let id = packet.get_id();

  let size_byte_len = buf.write_var_int(size as u64)?;
  buf.write_var_int(id)?;

  return Ok(size + size_byte_len);
}

impl Packet for StatusPacketToClient {
  type Output = StatusPacketToClient;

  fn get_id(&self) -> u64 {
    match self {
      StatusPacketToClient::Status(_) => 0,
      StatusPacketToClient::Ping(_) => 1,
    }
  }

  fn size_of(&self) -> Result<usize> {
    match self {
      StatusPacketToClient::Status(Status { status_data, .. }) => {
        let len = status_data.len();
        return Ok(var_int_len(self.get_id() as u64) + var_int_len(len as u64) + len);
      }
      StatusPacketToClient::Ping(_) => {
        return Ok(var_int_len(self.get_id() as u64) + std::mem::size_of::<u64>());
      }
    }
  }

  fn serialize(&self, buf: &mut impl Write) -> Result<usize> {
    let size = serialize_pre(buf, self)?;
    match self {
      StatusPacketToClient::Status(Status { status_data }) => {
        buf.write_string(status_data)?;
      }
      StatusPacketToClient::Ping(ping) => {
        buf.write_u64::<BigEndian>(*ping)?;
      }
    }

    Ok(size)
  }

  fn deserialize(_buf: &mut impl Read) -> Result<StatusPacketToClient> {
    unreachable!("ToClient packets should not be deserialized!");
  }
}
