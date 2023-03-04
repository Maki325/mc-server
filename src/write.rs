use std::io::{self, Result};

pub const SEGMENT_BITS: u64 = 0x7F;
pub const CONTINUE_BIT: u64 = 0x80;

pub fn var_int_len(mut var_int: u64) -> usize {
  let mut bytes = 0;
  loop {
    bytes += 1;
    if (var_int & (!SEGMENT_BITS)) == 0 {
      return bytes;
    }
    var_int >>= 7;
  }
}

pub trait WriteMCExt: io::Write {
  fn write_var_int(&mut self, mut var_int: u64) -> Result<usize> {
    let mut bytes = 0;
    loop {
      bytes += 1;
      if (var_int & (!SEGMENT_BITS)) == 0 {
        self.write_all(&[var_int as u8])?;
        return Ok(bytes);
      }

      self.write_all(&[((var_int & SEGMENT_BITS) | CONTINUE_BIT) as u8])?;

      var_int >>= 7;
    }
  }

  fn write_string(&mut self, string: &String) -> Result<usize> {
    let len = string.len();
    let mut size = self.write_var_int(len as u64)?;
    size += self.write(string.as_bytes())?;

    Ok(size)
  }
}

impl<W: io::Write + ?Sized> WriteMCExt for W {}
