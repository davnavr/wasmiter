use crate::bytes::{Bytes, Result};

/// Allows reading bytes from a memory map.
///
/// # Safety
///
/// Although UB is easy if underlying storage is modified, **pretending** it doesn't happen should
/// be fine, since bytes will still be read no matter what (their values don't matter).
impl Bytes for memmap2::Mmap {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        <[u8] as Bytes>::read_at(self.as_ref(), offset, buffer)
    }

    fn length_at(&self, offset: u64) -> Result<u64> {
        <[u8] as Bytes>::length_at(self, offset)
    }
}
