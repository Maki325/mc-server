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
use std::{fmt::Display, net::SocketAddr};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug)]
pub enum State {
  Handshake,
  Status,
  Login,
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

  pub async fn hadle_handshake(&mut self) -> Result<()> {
    let packet = self.receive().await?;
    let handshake_packet = if let PacketToServer::Handshake(handshake) = packet {
      handshake
    } else {
      return Err(Error::UnexpectedPacket(packet, "PacketToServer::Handshake"));
    };
    println!("handshake_packet: {:#?}", handshake_packet);

    self.state = handshake_packet.next_state;

    Ok(())
  }

  pub async fn handle_status(&mut self) -> Result<()> {
    let packet = self.receive().await?;
    let status_packet = if let PacketToServer::Status(status_packet) = packet {
      status_packet
    } else {
      return Err(Error::UnexpectedPacket(packet, "PacketToServer::Status"));
    };

    let (packet_to_send, should_close_connection) = match status_packet {
      StatusPacketToServer::Ping(time) => (
        PacketToClient::Status(StatusPacketToClient::Ping(time)),
        true,
      ),
      StatusPacketToServer::Status => (
        PacketToClient::Status(StatusPacketToClient::Status(Status::new(
          "                  §cHello §b§lWORLD!".into(),
        )?)),
        false,
      ),
    };

    self.send(packet_to_send, true).await?;

    if should_close_connection {
      return Err(Error::ConnectionAborted(
        "Closing connection after Ping and Status packets are sent!",
      ));
    } else {
      Ok(())
    }
  }

  pub async fn tick(&mut self) -> Result<()> {
    println!("TICK!");
    let result = match &self.state {
      State::Handshake => self.hadle_handshake().await,
      State::Status => self.handle_status().await,
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

  pub async fn send(&mut self, packet: PacketToClient, flush: bool) -> Result<()> {
    packet.serialize(&mut self.stream).await?;

    if flush {
      self.stream.flush().await?;
    }

    Ok(())
  }

  pub async fn receive(&mut self) -> Result<PacketToServer> {
    println!("Receive connection!");
    let buf = &mut self.stream;
    return Ok(match self.state {
      State::Handshake => {
        PacketToServer::Handshake(HandshakePacketToServer::deserialize(buf).await?)
      }
      State::Status => PacketToServer::Status(StatusPacketToServer::deserialize(buf).await?),
      _ => unimplemented!(),
    });
  }
}
