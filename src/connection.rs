use crate::{
  error::Error,
  packet::to_server::PacketToServer,
  packet::{
    to_client::{
      status::{Status, StatusPacketToClient},
      PacketToClient,
    },
    to_server::{handshake::HandshakePacketToServer, status::StatusPacketToServer},
    Packet,
  },
  result::Result,
};
use std::{
  fmt::Display,
  io::Write,
  net::{SocketAddr, TcpStream},
};

#[derive(Debug)]
pub enum State {
  Handshake,
  Status,
  Login,
  // Play,
}

impl Display for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        State::Handshake => "Handshake",
        State::Status => "Status",
        State::Login => "Login",
      }
    )
  }
}

pub struct Connection {
  pub stream: TcpStream,
  pub address: SocketAddr,
  state: State,
  skipped_ticks: u32,
}

const MAX_TICKS_SKIPED: u32 = 20 * 5; // 5 Seconds in MC Ticks

impl Connection {
  pub fn new(stream: TcpStream, address: SocketAddr) -> Connection {
    Connection {
      stream,
      address,
      state: State::Handshake,
      skipped_ticks: 0,
    }
  }

  pub fn hadle_handshake(&mut self) -> Result<()> {
    let packet = self.receive()?;
    println!("hadle_handshake: {:#?}", packet);
    let handshake_packet = if let PacketToServer::Handshake(handshake) = packet {
      handshake
    } else {
      println!("ERR");

      return Err(Error::UnexpectedPacket(packet, "PacketToServer::Handshake"));
    };
    println!("handshake_packet: {:#?}", handshake_packet);

    self.state = handshake_packet.next_state;

    Ok(())
  }

  pub fn handle_status(&mut self) -> Result<()> {
    let packet = self.receive()?;
    println!("handle_status: {:#?}", packet);
    let status_packet = if let PacketToServer::Status(status_packet) = packet {
      status_packet
    } else {
      return Err(Error::UnexpectedPacket(packet, "PacketToServer::Status"));
    };

    let (packet_to_send, should_close_connection) = match status_packet {
      StatusPacketToServer::Ping(time) => {
        println!("PING {}", time);
        (
          PacketToClient::Status(StatusPacketToClient::Ping(time)),
          true,
        )
      }
      StatusPacketToServer::Status => {
        println!("STATUS");
        // StatusPacketToClient::Status(Status::new("Hello World!".into())?)
        (
          PacketToClient::Status(StatusPacketToClient::Status(Status::new(
            "                  §cHello §b§lWORLD!".into(),
          )?)),
          false,
        )
      }
    };

    self.send(packet_to_send, true)?;

    if should_close_connection {
      return Err(Error::ConnectionAborted(
        "Closing connection after Ping and Status packets are sent!",
      ));
    } else {
      Ok(())
    }
  }

  pub fn tick(&mut self) -> Result<()> {
    // let packet = self.receive()?;

    let result = match &self.state {
      State::Status => self.handle_status(),
      _ => unimplemented!(),
    };

    return match result {
      Ok(value) => {
        self.skipped_ticks = 0;
        Ok(value)
      }
      Err(e) => {
        self.skipped_ticks += 1;
        if self.skipped_ticks >= MAX_TICKS_SKIPED {
          Err(Error::TimedOut)
        } else {
          Err(e)
        }
      }
    };
  }

  pub fn send(&mut self, packet: PacketToClient, flush: bool) -> Result<()> {
    packet.serialize(&mut self.stream)?;

    if flush {
      self.stream.flush()?;
    }

    Ok(())
  }

  pub fn receive(&mut self) -> Result<PacketToServer> {
    let buf = &mut self.stream;
    println!("self.state: {}", self.state);
    return Ok(match self.state {
      State::Handshake => PacketToServer::Handshake(HandshakePacketToServer::deserialize(buf)?),
      State::Status => PacketToServer::Status(StatusPacketToServer::deserialize(buf)?),
      _ => unimplemented!(),
    });
  }
}
