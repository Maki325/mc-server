mod read;
mod write;

pub use read::ReadMCExt;
pub use write::WriteMCExt;

const SEGMENT_BITS: u64 = 0x7F;
const CONTINUE_BIT: u64 = 0x80;

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
