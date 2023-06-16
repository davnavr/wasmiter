use crate::{
    index::Index,
    input::Input,
    parser::{self, name::Name, ResultExt as _},
};

/// A [**nameassoc**](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps)
/// associates an [`Index`] with a [`Name`].
#[derive(Clone, Copy)]
pub struct NameAssoc<N: Index, I: Input> {
    index: N,
    name: Name<I>,
}

impl<N: Index, I: Input> NameAssoc<N, I> {
    /// Parses a [`NameAssoc`].
    pub fn parse(offset: &mut u64, input: I) -> parser::Result<Self> {
        Ok(Self {
            index: crate::component::index(offset, &input).context("index of nameassoc pair")?,
            name: parser::name::parse(offset, input).context("name of nameassoc pair")?,
        })
    }

    /// Gets the index.
    #[inline]
    pub fn index(&self) -> N {
        self.index
    }

    /// Gets the name.
    #[inline]
    pub fn name(&self) -> &Name<I> {
        &self.name
    }
}

impl<N: Index, I: Clone + Input> NameAssoc<N, &I> {
    pub(super) fn dereferenced(&self) -> NameAssoc<N, I> {
        NameAssoc {
            index: self.index,
            name: self.name.really_cloned(),
        }
    }
}

impl<N: Index, I: Input> core::fmt::Debug for NameAssoc<N, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NameAssoc")
            .field("index", &self.index)
            .field("name", &self.name)
            .finish()
    }
}
