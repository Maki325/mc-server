use std::io::ErrorKind;

use async_trait::async_trait;
use byteorder::ByteOrder;
use tokio::io::AsyncReadExt;

use crate::{error::Error, result::Result};

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

#[async_trait]
pub trait ReadMCExt: AsyncReadExt {
  /// Read the exact number of bytes required to fill `buf`.
  /// But opposed to `read_exact`, it'll wait untill the `buf` is full
  /// Which means it could wait forever, if the stream doesn't send any bytes
  async fn read_exact_blocking(&mut self, buf: &mut [u8]) -> Result<()>
  where
    Self: Unpin,
  {
    let mut cursor = 0;
    loop {
      let result = self.read(&mut buf[cursor..]).await;
      if let Ok(size) = result {
        cursor += size;
      }
      if let Err(e) = result {
        if e.kind() != ErrorKind::Interrupted {
          return Err(e.into());
        }
      }
      if cursor >= buf.len() {
        break;
      }
    }

    Ok(())
  }

  #[inline]
  async fn read_u8(&mut self) -> Result<u8>
  where
    Self: Unpin,
  {
    let mut buf = [0; 1];
    self.read_exact_blocking(&mut buf).await?;
    Ok(buf[0])
  }

  #[inline]
  async fn read_u16<T: ByteOrder>(&mut self) -> Result<u16>
  where
    Self: Unpin,
  {
    let mut buf = [0; 2];
    self.read_exact_blocking(&mut buf).await?;
    Ok(T::read_u16(&buf))
  }

  #[inline]
  async fn read_u64<T: ByteOrder>(&mut self) -> Result<u64>
  where
    Self: Unpin,
  {
    let mut buf = [0; 8];
    self.read_exact_blocking(&mut buf).await?;
    Ok(T::read_u64(&buf))
  }

  async fn read_var_int(&mut self, blocking: bool) -> Result<u64>
  where
    Self: Unpin,
  {
    let mut value: u64 = 0;
    let mut position = 0;

    loop {
      let current_byte = {
        if blocking {
          ReadMCExt::read_u8(self).await
        } else {
          match AsyncReadExt::read_u8(self).await {
            Ok(value) => Ok(value),
            Err(e) => Err(e.into()),
          }
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

  async fn get_packet_size(&mut self) -> Result<u64>
  where
    Self: Unpin,
  {
    return match self.read_var_int(false).await {
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

  async fn read_string(&mut self) -> Result<String>
  where
    Self: Unpin,
  {
    let len = self.read_var_int(true).await?;

    let mut data: Vec<u8> = vec![0; len as usize];
    self.read_exact_blocking(&mut data[..]).await?;

    return String::from_utf8(data).map_err(|e| std::io::Error::new(ErrorKind::Other, e).into());
  }
}

impl<R: AsyncReadExt + ?Sized> ReadMCExt for R {}
