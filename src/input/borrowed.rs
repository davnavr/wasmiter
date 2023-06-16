/// Trait for creating a clone of an [`Input`](crate::input::Input) by borrowing it.
pub trait BorrowInput {
    /// The type of the clone with the borrowed [`Input`](crate::input::Input).
    type Borrowed<'a> where Self: 'a;

    /// Borrows the underlying [`Input`](crate::input::Input).
    fn borrow_input(&self) -> Self::Borrowed<'_>;
}
