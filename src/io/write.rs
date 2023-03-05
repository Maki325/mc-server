use crate::result::Result;
use async_trait::async_trait;
use byteorder::ByteOrder;
use tokio::io::AsyncWriteExt;

const SEGMENT_BITS: u64 = super::SEGMENT_BITS as u64;
const CONTINUE_BIT: u64 = super::CONTINUE_BIT as u64;

#[async_trait]
pub trait WriteMCExt: AsyncWriteExt {
  async fn write_var_int(&mut self, mut var_int: u64) -> Result<usize>
  where
    Self: Unpin,
  {
    let mut bytes = 0;
    loop {
      bytes += 1;
      if (var_int & (!SEGMENT_BITS)) == 0 {
        self.write_all(&[var_int as u8]).await?;
        return Ok(bytes);
      }

      self
        .write_all(&[((var_int & SEGMENT_BITS) | CONTINUE_BIT) as u8])
        .await?;

      var_int >>= 7;
    }
  }

  async fn write_string(&mut self, string: &String) -> Result<usize>
  where
    Self: Unpin,
  {
    let len = string.len();
    let mut size = self.write_var_int(len as u64).await?;
    size += self.write(string.as_bytes()).await?;

    Ok(size)
  }

  #[inline]
  async fn write_u64<T: ByteOrder>(&mut self, n: u64) -> Result<()>
  where
    Self: Unpin,
  {
    let mut buf = [0; 8];
    T::write_u64(&mut buf, n);
    self.write_all(&buf).await?;

    Ok(())
  }
}

impl<W: AsyncWriteExt + ?Sized> WriteMCExt for W {}
