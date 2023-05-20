use crate::bytes::Bytes;
use core::fmt::{Debug, Write};

/// Provides a [`Debug`] implementation for [`Bytes`]
pub struct DebugBytes<B: Bytes> {
    bytes: B,
}

impl<B: Bytes> From<B> for DebugBytes<B> {
    #[inline]
    fn from(bytes: B) -> Self {
        Self { bytes }
    }
}

impl<B: Bytes> Debug for DebugBytes<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();

        let mut offset = 0;
        let mut buffer = [0u8; 512];

        struct SixteenBytes<'a>(&'a [u8]);

        impl Debug for SixteenBytes<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                for (i, b) in self.0.iter().enumerate() {
                    if matches!(i, 4 | 8 | 12) {
                        f.write_char('_')?;
                    }

                    if i == 8 {
                        f.write_char('_')?;
                    }

                    Debug::fmt(&format_args!("{b:02X}"), f)?;
                }

                Ok(())
            }
        }

        loop {
            match self.bytes.read(&mut offset, &mut buffer) {
                Ok([]) => break,
                Ok(bytes) => {
                    for slice in bytes.chunks(16) {
                        list.entry(&SixteenBytes(slice));
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
