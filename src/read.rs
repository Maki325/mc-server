use std::io::{self, ErrorKind, Result};

use byteorder::ByteOrder;

pub trait ReadMCExt: io::Read {
  /// Read the exact number of bytes required to fill `buf`.
  /// But opposed to `read_exact`, it'll wait untill the `buf` is full
  /// Which means it could wait forever, if the stream doesn't send any bytes
  fn read_exact_blocking(&mut self, buf: &mut [u8]) -> Result<()> {
    let mut cursor = 0;
    loop {
      let result = self.read(&mut buf[cursor..]);
      if let Ok(size) = result {
        cursor += size;
      }
      if let Err(e) = result {
        if e.kind() != ErrorKind::Interrupted {
          return Err(e);
        }
      }
      if cursor >= buf.len() {
        break;
      }
    }

    Ok(())
  }

  #[inline]
  fn read_u8(&mut self) -> Result<u8> {
    let mut buf = [0; 1];
    self.read_exact_blocking(&mut buf)?;
    Ok(buf[0])
  }

  #[inline]
  fn read_u16<T: ByteOrder>(&mut self) -> Result<u16> {
    let mut buf = [0; 2];
    self.read_exact_blocking(&mut buf)?;
    Ok(T::read_u16(&buf))
  }

  #[inline]
  fn read_u64<T: ByteOrder>(&mut self) -> Result<u64> {
    let mut buf = [0; 8];
    self.read_exact_blocking(&mut buf)?;
    Ok(T::read_u64(&buf))
  }
}

impl<R: io::Read + ?Sized> ReadMCExt for R {}
