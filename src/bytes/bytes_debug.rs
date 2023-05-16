use crate::bytes::Bytes;
use core::fmt::Debug;

/// Provides a [`Debug`] implementation for [`Bytes`]
pub struct BytesDebug<B: Bytes> {
    bytes: B,
}

impl<B: Bytes> From<B> for BytesDebug<B> {
    #[inline]
    fn from(bytes: B) -> Self {
        Self { bytes }
    }
}

impl<B: Bytes> Debug for BytesDebug<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();

        let mut offset = 0;
        let mut buffer = [0u8; 512];

        loop {
            let result = self.bytes.read(&mut offset, &mut buffer);

            match result {
                Ok([]) => break,
                Ok(bytes) => {
                    for b in bytes.iter().copied() {
                        list.entry(&format_args!("{b:02X}"));
                    }
                }
                Err(e) => {
                    list.entry(&e);
                    break;
                }
            }
        }

        list.finish()
    }
}
