use crate::{connection::Connection, error::Error, result::Result};
use std::{
  io::ErrorKind,
  net::TcpListener,
  thread,
  time::{Duration, Instant},
};

mod connection;
mod error;
mod packet;
mod read;
mod result;
mod write;

fn main() -> Result<()> {
  const TICK_SLEEP_TIME: u64 = 1000 / 20;

  let listener = TcpListener::bind("127.0.0.1:25565").unwrap();
  listener.set_nonblocking(true)?;

  let mut connections: Vec<Connection> = vec![];

  let mut sleep_subtract: u64 = 0;
  loop {
    let tick_time = Instant::now();

    connections.retain_mut(|connection| {
      if let Err(error) = match connection.tick() {
        Err(error) => {
          match error {
            Error::NoPacketToReceive => return true,
            Error::ConnectionAborted(..) | Error::TimedOut => {
              return false;
            }
            _ => {}
          }
          Err(error)
        }
        Ok(value) => Ok(value),
      } {
        println!("Error: {}", error);
      }

      return true;
    });
    loop {
      let result = listener.accept();
      if let Ok((stream, address)) = result {
        println!("Stream: {:#?}, address: {:#?}", stream, address);
        println!("stream.ttl(): {}", stream.ttl()?);
        let mut connection = Connection::new(stream, address);
        // while let Ok(byte) = connection.stream.read_u8() {
        //   println!("byte: {}", byte);
        // }
        // continue;

        if let Err(err) = connection.hadle_handshake() {
          println!("Error! {}", err);
          // if err.kind() == ErrorKind::Other {
          //   println!("Other error! {}", err);
          // }
        };
        connections.push(connection);
        continue;
      }
      if let Err(err) = result {
        if err.kind() == ErrorKind::WouldBlock {
          // println!("SKIP!");
          break;
        }
        return Err(Error::IO(err));
      }
    }

    let sleep_time = sleep_subtract + tick_time.elapsed().as_millis() as u64;
    if sleep_time >= TICK_SLEEP_TIME {
      sleep_subtract = sleep_time - TICK_SLEEP_TIME;
    } else {
      sleep_subtract = 0;
      // println!("Sleep time: {}", TICK_SLEEP_TIME - sleep_time);
      thread::sleep(Duration::from_millis(TICK_SLEEP_TIME - sleep_time));
    }
  }
}