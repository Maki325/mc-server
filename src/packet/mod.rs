pub mod to_client;
pub mod to_server;

use crate::result::Result;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[async_trait]
pub trait Packet {
  type Output;

  async fn deserialize<R>(buf: &mut R) -> Result<Self::Output>
  where
    R: AsyncReadExt + std::marker::Unpin + std::marker::Send;

  async fn serialize<W>(&self, buf: &mut W) -> Result<usize>
  where
    W: AsyncWriteExt + std::marker::Unpin + std::marker::Send;

  fn get_id(&self) -> u64;
  fn size_of(&self) -> Result<usize>;
}
