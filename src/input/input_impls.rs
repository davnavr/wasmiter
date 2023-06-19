use crate::input::{self, Input, Result};

impl Input for [u8] {
    #[inline]
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        let start = usize::try_from(offset).unwrap_or(usize::MAX);
        if let Some(source) = self.get(start..) {
            let copy_amount = core::cmp::min(source.len(), buffer.len());
            let destination = &mut buffer[..copy_amount];
            destination.copy_from_slice(&source[..copy_amount]);
            Ok(destination)
        } else {
            Err(input::out_of_bounds(
                offset,
                self.len()
                    .checked_sub(start)
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
impl Input for memmap2::Mmap {
    #[inline]
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
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
            fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
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
            fn read<'b>(&self, offset: &mut u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
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
