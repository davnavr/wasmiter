use crate::{
    bytes::Bytes,
    parser::name::{InvalidCodePoint, Name, NameError},
};

#[derive(Clone, Copy, Default)]
struct CharsBuffer {
    /// - Lower 4 bits contain number of saved bytes in `buffer`.
    /// - Upper 4 bits contain number of valid UTF-8 bytes in `buffer`.
    lengths: u8,
    /// Length of the error after `valid_len`.
    bad_sequence: Option<core::num::NonZeroU8>,
    /// # Safety
    ///
    /// The first `valid_len` bytes **must** be valid UTF-8.
    buffer: [u8; 15],
}

impl CharsBuffer {
    #[inline]
    fn saved_len(&self) -> u8 {
        self.lengths & 0xF
    }

    #[inline]
    fn valid_len(&self) -> u8 {
        self.lengths >> 4
    }

    fn valid(&self) -> &str {
        let valid_bytes = &self.buffer[0..usize::from(self.valid_len())];
        if cfg!(debug_assertions) {
            core::str::from_utf8(valid_bytes).unwrap()
        } else {
            // Safety: it is an invariant that valid_len bytes are valid
            unsafe { core::str::from_utf8_unchecked(valid_bytes) }
        }
    }

    fn advance(&mut self, amount: u8) {
        debug_assert!(amount <= self.saved_len());

        self.lengths = (self.lengths & 0xF0) | ((self.saved_len() - amount) & 0xF);

        if self.valid_len() > 0 {
            debug_assert!(amount <= self.valid_len());
            self.lengths = ((self.valid_len() - amount) << 4) | (self.lengths & 0xF);
        }

        let start = usize::from(amount);
        let new_length = start + usize::from(self.saved_len());
        self.buffer.copy_within(start..new_length, 0);
    }

    fn take_char(&mut self) -> Result<Option<char>, NameError> {
        if self.valid_len() > 0 {
            let mut chars = self.valid().chars();
            let original_len = chars.as_str().len();

            let c = if cfg!(debug_assertions) {
                chars.next().unwrap()
            } else {
                // Safety: check above ensures string is not empty
                unsafe { chars.next().unwrap_unchecked() }
            };

            // Skip the amount of bytes that were read
            self.advance((original_len - chars.as_str().len()) as u8);

            Ok(Some(c))
        } else if let Some(bad) = self.bad_sequence.take() {
            let bad_len = usize::from(bad.get());
            let mut bytes = [0u8; 4];
            bytes[0..bad_len].copy_from_slice(&self.buffer[0..bad_len]);

            // Skip the invalid sequence
            self.advance(bad.get());

            Err(NameError::BadBytes(InvalidCodePoint { length: bad, bytes }))
        } else {
            Ok(None)
        }
    }

    fn fill(
        &mut self,
        offset: &mut u64,
        length: &mut u32,
        bytes: &impl Bytes,
    ) -> Result<(), NameError> {
        // take_char would return Ok(Some) or Err for the same conditions here
        if self.valid_len() > 0 || self.bad_sequence.is_some() {
            return Ok(());
        }

        let saved_length = usize::from(self.saved_len());
        if saved_length < self.buffer.len() {
            let remaining = self.buffer.len() - saved_length;
            let actual_remaining = core::cmp::min(remaining, *length as usize);
            let result = bytes
                .read(offset, &mut self.buffer[saved_length..][..actual_remaining])
                .map(|buf| buf.len() as u8);

            match result {
                Ok(filled) => {
                    self.lengths = (self.lengths & 0xF0) | ((saved_length as u8 + filled) & 0xF);
                    *length -= u32::from(filled);
                }
                Err(e) => {
                    *length = 0;
                    self.lengths = 0;
                    self.bad_sequence = None;
                    return Err(NameError::BadInput(e));
                }
            }
        }

        debug_assert_eq!(self.valid_len(), 0);

        let valid_len = match core::str::from_utf8(&self.buffer[0..usize::from(self.saved_len())]) {
            Ok(s) => s.len(),
            Err(e) => {
                if let Some(bad_len) = e.error_len() {
                    self.bad_sequence = Some(
                        u8::try_from(bad_len)
                            .ok()
                            .and_then(core::num::NonZeroU8::new)
                            .unwrap(),
                    );
                }

                e.valid_up_to()
            }
        };

        self.lengths = ((valid_len as u8) << 4) | (self.lengths & 0xF);
        Ok(())
    }
}

/// An iterator over the [`char`]s of a [`Name`].
///
/// See the documentation for [`Name::chars()`] for more information.
#[derive(Clone, Copy)]
#[must_use]
pub struct Chars<B: Bytes> {
    name: Name<B>,
    buffer: CharsBuffer,
}

impl<B: Bytes> Chars<B> {
    pub(super) fn new(name: Name<B>) -> Self {
        Self {
            name,
            buffer: Default::default(),
        }
    }

    pub(super) fn borrowed(&self) -> Chars<&B> {
        Chars {
            name: self.name.borrowed(),
            buffer: self.buffer,
        }
    }

    fn next_inner(&mut self) -> Result<Option<char>, NameError> {
        if self.name.length == 0 && self.buffer.saved_len() == 0 {
            return Ok(None);
        }

        self.buffer.fill(
            &mut self.name.offset,
            &mut self.name.length,
            &self.name.bytes,
        )?;

        self.buffer.take_char()
    }
}

impl<B: Bytes> Iterator for Chars<B> {
    type Item = Result<char, NameError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner().transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max = usize::from(self.buffer.saved_len()) + (self.name.length as usize);
        (core::cmp::min(1, max), Some(max))
    }
}

impl<B: Bytes> core::iter::FusedIterator for Chars<B> {}

/// An iterator over the [`char`]s of a [`Name`] that substitutes invalid byte sequences and other errors with [`char::REPLACEMENT_CHARACTER`].
///
/// See the documentation for [`Name::chars_lossy()`] for more information.
#[derive(Clone, Copy)]
#[must_use]
pub struct CharsLossy<B: Bytes> {
    inner: Chars<B>,
}

impl<B: Bytes> CharsLossy<B> {
    #[inline]
    pub(super) fn new(inner: Chars<B>) -> CharsLossy<B> {
        Self { inner }
    }

    #[inline]
    pub(super) fn borrowed(&self) -> CharsLossy<&B> {
        CharsLossy::new(self.inner.borrowed())
    }
}

impl<B: Bytes> Iterator for CharsLossy<B> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        Some(match self.inner.next()? {
            Ok(c) => c,
            Err(_) => char::REPLACEMENT_CHARACTER,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<B: Bytes> core::iter::FusedIterator for CharsLossy<B> {}
