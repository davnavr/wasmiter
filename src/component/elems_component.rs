use crate::{
    component::{self, IndexVector},
    index::{self, TableIdx},
    input::{BorrowInput, CloneInput, HasInput, Input},
    instruction_set::InstructionSequence,
    parser::{self, Offset, Result, ResultExt as _, Vector},
};
use core::fmt::{Debug, Formatter};

/// Represents a vector of expressions that evaluate to references in an
/// [element segment](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments).
pub struct ElementExpressions<O: Offset, I: Input> {
    expressions: Vector<O, I>,
}

impl<O: Offset, I: Input> From<Vector<O, I>> for ElementExpressions<O, I> {
    #[inline]
    fn from(expressions: Vector<O, I>) -> Self {
        Self { expressions }
    }
}

impl<O: Offset, I: Input> ElementExpressions<O, I> {
    fn new(offset: O, input: I) -> Result<Self> {
        Vector::parse(offset, input)
            .context("at start of element segment expressions")
            .map(Self::from)
    }

    /// Parses the next expression.
    pub fn next<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut InstructionSequence<&mut u64, &I>) -> Result<T>,
    {
        self.expressions
            .advance(|offset, input| {
                let mut offset_cell = *offset;
                let mut expression = InstructionSequence::new(&mut offset_cell, input);
                let result = f(&mut expression)?;
                let (_, final_offset) = expression.finish()?;
                *offset = *final_offset;
                Result::Ok(result)
            })
            .transpose()
            .context("could not parse element segment expression")
    }

    fn finish(mut self) -> Result<()> {
        while self.next(|_| Result::Ok(()))?.is_some() {}
        Ok(())
    }
}

impl<O: Offset, I: Input> HasInput<I> for ElementExpressions<O, I> {
    #[inline]
    fn input(&self) -> &I {
        self.expressions.input()
    }
}

impl<'a, O: Offset, I: Input + 'a> BorrowInput<'a, I> for ElementExpressions<O, I> {
    type Borrowed = ElementExpressions<u64, &'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.expressions.borrow_input().into()
    }
}

impl<O: Offset, I: Input> Debug for ElementExpressions<O, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut borrowed = self.borrow_input();
        let mut list = f.debug_list();
        loop {
            let result = borrowed.next(|instructions| {
                list.entry(&Result::Ok(instructions));
                Ok(())
            });

            match result {
                Ok(Some(())) => (),
                Ok(None) => break,
                Err(e) => {
                    list.entry(&Result::<()>::Err(e));
                    break;
                }
            }
        }
        list.finish()
    }
}

/// Represents the references within an
/// [element segment](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments).
pub enum ElementInit<O: Offset, I: Input> {
    /// A vector of functions to create `funcref` elements from.
    Functions(IndexVector<index::FuncIdx, O, I>),
    /// A vector of expressions that evaluate to references.
    Expressions(crate::types::RefType, ElementExpressions<O, I>),
}

impl<O: Offset, I: Input> ElementInit<O, I> {
    fn finish(self) -> Result<()> {
        match self {
            Self::Functions(functions) => {
                functions.finish()?;
            }
            Self::Expressions(_, expressions) => expressions.finish()?,
        }
        Ok(())
    }
}

impl<O: Offset, I: Input> Debug for ElementInit<O, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Functions(functions) => f.debug_tuple("Functions").field(functions).finish(),
            Self::Expressions(rtype, expressions) => f
                .debug_struct("Expressions")
                .field("type", rtype)
                .field("values", expressions)
                .finish(),
        }
    }
}

/// Specifies a kind of [element segment](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments).
pub enum ElementMode<O: Offset, I: Input> {
    /// A **passive** element segment's elements are copied to a table using the
    /// [`table.init`](crate::instruction_set::Instruction::TableInit) instruction.
    Passive,
    /// An **active** element segment copies elements into the specified table, starting at the
    /// expressed offset specified by an expression, during
    /// [instantiation](https://webassembly.github.io/spec/core/exec/modules.html#exec-instantiation)
    /// of the module.
    Active(TableIdx, InstructionSequence<O, I>),
    /// A **declarative** data segment cannot be used at runtime. It can be used as a hint to
    /// indicate that references to the given elements will be used in code later in the module.
    Declarative,
}

impl<O: Offset, I: Input> ElementMode<O, I> {
    fn finish(self) -> Result<()> {
        match self {
            Self::Passive | Self::Declarative => (),
            Self::Active(_, instructions) => {
                instructions.finish()?;
            }
        }
        Ok(())
    }
}

impl<O: Offset, I: Input> Debug for ElementMode<O, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Passive => f.debug_tuple("Passive").finish(),
            Self::Declarative => f.debug_tuple("Declarative").finish(),
            Self::Active(table, offset) => f
                .debug_struct("Active")
                .field("table", &table)
                .field("offset", offset)
                .finish(),
        }
    }
}

/// Represents the
/// [**elems** component](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments) of a
/// WebAssembly module, stored in and parsed from the
/// [*element section*](https://webassembly.github.io/spec/core/binary/modules.html#element-section).
#[derive(Clone, Copy)]
pub struct ElemsComponent<I: Input> {
    elements: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for ElemsComponent<I> {
    #[inline]
    fn from(elements: Vector<u64, I>) -> Self {
        Self { elements }
    }
}

fn elem_kind<I: Input>(offset: &mut u64, input: I) -> Result<()> {
    #[inline(never)]
    #[cold]
    fn bad_kind(kind: u8) -> parser::Error {
        parser::Error::new(parser::ErrorKind::BadElementKind(kind))
    }

    match parser::one_byte_exact(offset, input).context("elemkind")? {
        0 => Ok(()),
        bad => Err(bad_kind(bad)),
    }
}

impl<I: Input> ElemsComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *element section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, input: I) -> Result<Self> {
        Vector::parse(offset, input)
            .context("at start of element section")
            .map(Self::from)
    }

    /// Parses the next element segment in the section.
    pub fn parse<Y, Z, M, E>(&mut self, mode_f: M, init_f: E) -> Result<Option<Z>>
    where
        M: FnOnce(&mut ElementMode<&mut u64, &I>) -> Result<Y>,
        E: FnOnce(Y, &mut ElementInit<&mut u64, &I>) -> Result<Z>,
    {
        self.elements
            .advance(|offset, bytes| {
                let start = *offset;
                let segment_kind =
                    parser::leb128::u32(offset, bytes).context("element segment mode")?;

                let mut init;
                let init_arg: Y;
                match segment_kind {
                    0 => {
                        let mut offset_copy = *offset;
                        let mut mode = ElementMode::Active(
                            TableIdx::from(0u8),
                            InstructionSequence::new(&mut offset_copy, bytes),
                        );

                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?;
                        *offset = offset_copy;

                        init = ElementInit::Functions(
                            IndexVector::parse(offset, bytes)
                                .context("function references in active element segment")?,
                        );
                    }
                    1 => {
                        let mut mode = ElementMode::Passive;
                        elem_kind(offset, bytes)?;
                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?; // Does nothing
                        init = ElementInit::Functions(
                            IndexVector::parse(offset, bytes)
                                .context("function references in passive element segment")?,
                        );
                    }
                    2 => {
                        let mut offset_copy = *offset;
                        let mut mode = ElementMode::Active(
                            component::index(&mut offset_copy, bytes)?,
                            InstructionSequence::new(&mut offset_copy, bytes),
                        );

                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?;
                        *offset = offset_copy;

                        elem_kind(offset, bytes)?;
                        init = ElementInit::Functions(
                            IndexVector::parse(offset, bytes)
                                .context("function references in active element segment")?,
                        );
                    }
                    3 => {
                        let mut mode = ElementMode::Declarative;
                        elem_kind(offset, bytes)?;
                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?; // Does nothing
                        init = ElementInit::Functions(
                            IndexVector::parse(offset, bytes)
                                .context("function references in declarative element segment")?,
                        );
                    }
                    4 => {
                        let mut offset_copy = *offset;
                        let mut mode = ElementMode::Active(
                            TableIdx::from(0u8),
                            InstructionSequence::new(&mut offset_copy, bytes),
                        );

                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?;
                        *offset = offset_copy;

                        init = ElementInit::Expressions(
                            crate::types::RefType::Func,
                            ElementExpressions::new(offset, bytes)
                                .context("expressions in active element segment")?,
                        );
                    }
                    5 => {
                        let rtype = component::ref_type(offset, bytes)?;
                        let mut mode = ElementMode::Passive;
                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?; // Does nothing
                        init = ElementInit::Expressions(
                            rtype,
                            ElementExpressions::new(offset, bytes)
                                .context("expressions in passive element segment")?,
                        );
                    }
                    6 => {
                        let mut offset_copy = *offset;
                        let mut mode = ElementMode::Active(
                            component::index(&mut offset_copy, bytes)?,
                            InstructionSequence::new(&mut offset_copy, bytes),
                        );

                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?;
                        *offset = offset_copy;

                        let rtype = component::ref_type(offset, bytes)?;
                        init = ElementInit::Expressions(
                            rtype,
                            ElementExpressions::new(offset, bytes)
                                .context("expressions in active element segment")?,
                        );
                    }
                    7 => {
                        let rtype = component::ref_type(offset, bytes)?;
                        let mut mode = ElementMode::Declarative;
                        init_arg = mode_f(&mut mode)?;
                        mode.finish()?; // Does nothing
                        init = ElementInit::Expressions(
                            rtype,
                            ElementExpressions::new(offset, bytes)
                                .context("expressions in declarative element segment")?,
                        );
                    }
                    _ => {
                        #[inline(never)]
                        #[cold]
                        fn unsupported_mode(offset: u64, mode: u32) -> parser::Error {
                            parser::Error::new(parser::ErrorKind::BadElementSegmentMode(mode))
                                .with_location_context("element segment entry", offset)
                        }

                        return Err(unsupported_mode(start, segment_kind));
                    }
                }

                let result = init_f(init_arg, &mut init)?;
                init.finish()?;
                Ok(result)
            })
            .transpose()
            .context("within element section")
    }

    /// Gets the expected remaining number of entires in the *element section* that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.elements.remaining_count()
    }
}

impl<I: Input> HasInput<I> for ElemsComponent<I> {
    #[inline]
    fn input(&self) -> &I {
        self.elements.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for ElemsComponent<I> {
    type Borrowed = ElemsComponent<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.elements.borrow_input().into()
    }
}

impl<I: Input> Debug for ElemsComponent<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut elems = self.borrow_input();
        let mut list = f.debug_list();

        struct Elem<'a, 'b, 'c, 'd, 'e, I: Input> {
            mode: ElementMode<u64, &'a I>,
            elements: &'b mut ElementInit<&'c mut u64, &'d &'e I>,
        }

        impl<I: Input> Debug for Elem<'_, '_, '_, '_, '_, I> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("ElementSegment")
                    .field("mode", &self.mode)
                    .field("elements", self.elements)
                    .finish()
            }
        }

        while elems.remaining_count() > 0 {
            let result = elems.parse(
                |mode| {
                    Ok(match mode {
                        ElementMode::Passive => ElementMode::Passive,
                        ElementMode::Declarative => ElementMode::Declarative,
                        ElementMode::Active(table, offset) => {
                            ElementMode::Active(*table, offset.clone_input())
                        }
                    })
                },
                |mode, elements| {
                    list.entry(&Elem { mode, elements });

                    Result::Ok(())
                },
            );

            if let Err(e) = result {
                list.entry(&Result::<()>::Err(e));
            }
        }
        list.finish()
    }
}
