use crate::{connection::Connection, error::Error, result::Result};
use std::{
  sync::mpsc,
  thread,
  time::{Duration, Instant},
};
use tokio::{net::TcpListener, task::spawn};

mod connection;
mod error;
mod io;
mod packet;
mod result;

const TICK_SLEEP_TIME: u64 = 1000 / 20;

#[tokio::main]
async fn main() -> Result<()> {
  let listener = TcpListener::bind("127.0.0.1:25565").await.unwrap();
  let mut connections: Vec<Connection> = vec![];
  let (tx, rx) = mpsc::channel::<Connection>();

  spawn(async move {
    loop {
      let (stream, address) = match listener.accept().await {
        Ok(result) => result,
        Err(err) => {
          println!("Error during accept accepting new connection! {}", err);
          continue;
        }
      };
      let mut connection = Connection::new(stream, address);

      if let Err(err) = connection.hadle_handshake().await {
        if let Error::NoPacketToReceive = err {
        } else {
          println!("Error during handshake! Disconnecting! {}", err);
          continue;
        }
      };

      if let Err(e) = tx.send(connection) {
        println!("TX Send error: {}", e);
      }
    }
  });

  let mut sleep_subtract: u64 = 0;
  loop {
    let tick_time = Instant::now();

    while let Ok(connection) = rx.try_recv() {
      connections.push(connection);
    }

    let mut idx = 0;
    while idx < connections.len() {
      if let Err(error) = match connections[idx].tick().await {
        Err(error) => {
          match error {
            Error::ConnectionAborted(..) | Error::TimedOut => {
              connections.remove(idx);
              continue;
            }
            Error::NoPacketToReceive => {
              idx += 1;
              continue;
            }
            _ => {}
          }
          Err(error)
        }
        Ok(value) => Ok(value),
      } {
        println!("Error: {}", error);
      }
      idx += 1;
    }

    let sleep_time = sleep_subtract + tick_time.elapsed().as_millis() as u64;
    if sleep_time >= TICK_SLEEP_TIME {
      sleep_subtract = sleep_time - TICK_SLEEP_TIME;
    } else {
      sleep_subtract = 0;
      thread::sleep(Duration::from_millis(TICK_SLEEP_TIME - sleep_time));
    }
  }
}
