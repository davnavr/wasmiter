use crate::input::Input;

/// Trait for types that have an [`Input`].
pub trait HasInput<I: Input> {
    /// Gets the underlying [`Input`].
    fn input(&self) -> &I;
}

/// Trait for creating a clone by borrowing an underlying [`Input`].
pub trait BorrowInput<'a, I: Input + 'a>: HasInput<I> {
    /// The type with the borrowed [`Input`].
    type Borrowed: HasInput<&'a I>;

    /// Creates a clone by borrows the underlying [`Input`].
    fn borrow_input(&'a self) -> Self::Borrowed;
}

/// Trait for cloning an underlying [`Input`].
pub trait CloneInput<'a, I: Clone + Input + 'a>: HasInput<&'a I> {
    /// The type with the cloned [`Input`].
    type Cloned: HasInput<I>;

    /// Clones the underlying [`Input`].
    fn clone_input(&self) -> Self::Cloned;
}
