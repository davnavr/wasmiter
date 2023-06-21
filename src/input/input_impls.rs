use crate::input::{self, Input, Result};

impl Input for [u8] {
    #[inline]
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let start = usize::try_from(offset).ok();

        if let Some(source) = start.and_then(|i| self.get(i..)) {
            let copy_amount = source.len().min(buffer.len());

            // Optimize calls to copy a single byte, just like <[u8] as std::io::Read>
            if copy_amount == 1 {
                buffer[0] = source[0];
            } else {
                buffer[..copy_amount].copy_from_slice(&source[..copy_amount]);
            }

            Ok(copy_amount)
        } else {
            Err(input::out_of_bounds(
                offset,
                start
                    .and_then(|i| self.len().checked_sub(i))
                    .and_then(|len| u64::try_from(len).ok()),
            ))
        }
    }

    #[inline]
    fn length_at(&self, offset: u64) -> Result<u64> {
        let length = usize::try_from(offset)
            .ok()
            .and_then(|start| self.len().checked_sub(start))
            .and_then(|len| u64::try_from(len).ok());

        if let Some(len) = length {
            Ok(len)
        } else {
            Err(input::out_of_bounds(offset, None))
        }
    }

    #[inline]
    fn try_eq_at(&self, offset: u64, bytes: &[u8]) -> Result<bool> {
        if let Some(input) = usize::try_from(offset)
            .ok()
            .and_then(|start| self.get(start..))
        {
            Ok(&input[..input.len().min(bytes.len())] == bytes)
        } else {
            Err(input::out_of_bounds(offset, None))
        }
    }
}

/// Allows reading bytes from a memory map.
///
/// # Safety
///
/// Although UB is easy if underlying storage is modified, **pretending** it doesn't happen should
/// be fine, since bytes will still be read no matter what (their values don't matter).
#[cfg(feature = "mmap")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "mmap")))]
impl Input for memmap2::Mmap {
    #[inline]
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        <[u8] as Input>::read_at(self.as_ref(), offset, buffer)
    }

    #[inline]
    fn length_at(&self, offset: u64) -> Result<u64> {
        <[u8] as Input>::length_at(self, offset)
    }

    #[inline]
    fn try_eq_at(&self, offset: u64, bytes: &[u8]) -> Result<bool> {
        <[u8] as Input>::try_eq_at(self, offset, bytes)
    }
}

macro_rules! delegated_input_impl {
    ($b:ident in $($implementor:ty $(,)?)+) => {$(
        impl<$b: Input + ?Sized> Input for $implementor {
            #[inline]
            fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
                <$b as Input>::read_at(self, offset, buffer)
            }

            #[inline]
            fn length_at(&self, offset: u64) -> Result<u64> {
                <$b as Input>::length_at(self, offset)
            }

            #[inline]
            fn try_eq_at(&self, offset: u64, bytes: &[u8]) -> Result<bool> {
                <$b as Input>::try_eq_at(self, offset, bytes)
            }

            #[inline]
            fn read_exact_at(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
                <$b as Input>::read_exact_at(self, offset, buffer)
            }

            #[inline]
            fn read(&self, offset: &mut u64, buffer: &mut [u8]) -> Result<usize> {
                <$b as Input>::read(self, offset, buffer)
            }

            #[inline]
            fn read_exact(&self, offset: &mut u64, buffer: &mut [u8]) -> Result<()> {
                <$b as Input>::read_exact(self, offset, buffer)
            }
        }
    )*};
}

delegated_input_impl!(I in &I);

#[cfg(feature = "alloc")]
delegated_input_impl!(I in alloc::sync::Arc<I>, alloc::rc::Rc<I>, alloc::boxed::Box<I>);
