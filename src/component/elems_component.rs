use crate::{
    bytes::Bytes,
    component,
    index::{self, TableIdx},
    instruction_set::InstructionSequence,
    parser::{self, Offset, Result, ResultExt, Vector},
};
use core::fmt::{Debug, Formatter};

/// Represents a vector of expressions that evaluate to references in an
/// [element segment](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments).
pub struct ElementExpressions<O: Offset, B: Bytes> {
    count: u32,
    offset: O,
    bytes: B,
}

impl<O: Offset, B: Bytes> ElementExpressions<O, B> {
    fn new(mut offset: O, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(offset.offset_mut(), &bytes)
                .context("element segment expression count")?,
            offset,
            bytes,
        })
    }

    fn next_inner<T, F>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(&mut InstructionSequence<&mut u64, &B>) -> Result<T>,
    {
        let mut offset_cell = self.offset.offset();
        let mut expression = InstructionSequence::new(&mut offset_cell, &self.bytes);
        let result = f(&mut expression)?;
        let (_, final_offset) = expression.finish()?;
        *self.offset.offset_mut() = *final_offset;
        Ok(result)
    }

    /// Parses the next expression.
    pub fn next<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut InstructionSequence<&mut u64, &B>) -> Result<T>,
    {
        if self.count == 0 {
            return Ok(None);
        }

        let result = self.next_inner(f);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }

    fn finish(mut self) -> Result<()> {
        while let Some(Ok(())) = self.next(|_| Ok(())).transpose() {}
        Ok(())
    }
}

impl<O: Offset, B: Bytes> Debug for ElementExpressions<O, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut borrowed = ElementExpressions {
            count: self.count,
            offset: self.offset.offset(),
            bytes: &self.bytes,
        };

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
pub enum ElementInit<O: Offset, B: Bytes> {
    /// A vector of functions to create `funcref` elements from.
    Functions(Vector<O, B, parser::SimpleParse<index::FuncIdx>>),
    /// A vector of expressions that evaluate to references.
    Expressions(Option<crate::types::RefType>, ElementExpressions<O, B>),
}

impl<O: Offset, B: Bytes> ElementInit<O, B> {
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

impl<O: Offset, B: Bytes> Debug for ElementInit<O, B> {
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
pub enum ElementMode<O: Offset, B: Bytes> {
    /// A **passive** element segment's elements are copied to a table using the
    /// [`table.init`](crate::instruction_set::Instruction::TableInit) instruction.
    Passive,
    /// An **active** element segment copies elements into the specified table, starting at the
    /// expressed offset specified by an expression, during
    /// [instantiation](https://webassembly.github.io/spec/core/exec/modules.html#exec-instantiation)
    /// of the module.
    Active(TableIdx, InstructionSequence<O, B>),
    /// A **declarative** data segment cannot be used at runtime. It can be used as a hint to
    /// indicate that references to the given elements will be used in code later in the module.
    Declarative,
}

impl<O: Offset, B: Bytes> ElementMode<O, B> {
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

impl<O: Offset, B: Bytes> Debug for ElementMode<O, B> {
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
pub struct ElemsComponent<B: Bytes> {
    count: u32,
    offset: u64,
    bytes: B,
}

fn elem_kind<B: Bytes>(offset: &mut u64, bytes: B) -> Result<()> {
    match parser::one_byte_exact(offset, bytes).context("elemkind")? {
        0 => Ok(()),
        bad => Err(crate::parser_bad_format!(
            "{bad:#04X} is not a valid elemkind"
        )),
    }
}

impl<B: Bytes> ElemsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *element section* of a module, starting
    /// at the specified `offset`.
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("element section count")?,
            bytes,
            offset,
        })
    }

    #[inline]
    fn parse_inner<Y, Z, M, I>(&mut self, mode_f: M, init_f: I) -> Result<Z>
    where
        M: FnOnce(&mut ElementMode<&mut u64, &B>) -> Result<Y>,
        I: FnOnce(Y, &mut ElementInit<&mut u64, &B>) -> Result<Z>,
    {
        let start = self.offset;
        let segment_kind =
            parser::leb128::u32(&mut self.offset, &self.bytes).context("element segment mode")?;

        let mut init;
        let init_arg: Y;
        match segment_kind {
            0 => {
                let mut mode = ElementMode::Active(
                    TableIdx::from(0u8),
                    InstructionSequence::new(&mut self.offset, &self.bytes),
                );

                init_arg = mode_f(&mut mode)?;
                mode.finish()?;
                init = ElementInit::Functions(
                    Vector::new(&mut self.offset, &self.bytes, Default::default())
                        .context("function references in active element segment")?,
                );
            }
            1 => {
                let mut mode = ElementMode::Passive;
                elem_kind(&mut self.offset, &self.bytes)?;
                init_arg = mode_f(&mut mode)?;
                mode.finish()?; // Does nothing
                init = ElementInit::Functions(
                    Vector::new(&mut self.offset, &self.bytes, Default::default())
                        .context("function references in passive element segment")?,
                );
            }
            2 => {
                let mut mode = ElementMode::Active(
                    component::index(&mut self.offset, &self.bytes)?,
                    InstructionSequence::new(&mut self.offset, &self.bytes),
                );

                init_arg = mode_f(&mut mode)?;
                mode.finish()?;
                elem_kind(&mut self.offset, &self.bytes)?;
                init = ElementInit::Functions(
                    Vector::new(&mut self.offset, &self.bytes, Default::default())
                        .context("function references in active element segment")?,
                );
            }
            3 => {
                let mut mode = ElementMode::Declarative;
                elem_kind(&mut self.offset, &self.bytes)?;
                init_arg = mode_f(&mut mode)?;
                mode.finish()?; // Does nothing
                init = ElementInit::Functions(
                    Vector::new(&mut self.offset, &self.bytes, Default::default())
                        .context("function references in declarative element segment")?,
                );
            }
            4 => {
                let mut mode = ElementMode::Active(
                    TableIdx::from(0u8),
                    InstructionSequence::new(&mut self.offset, &self.bytes),
                );

                init_arg = mode_f(&mut mode)?;
                mode.finish()?;
                init = ElementInit::Expressions(
                    None,
                    ElementExpressions::new(&mut self.offset, &self.bytes)
                        .context("expressions in active element segment")?,
                );
            }
            5 => {
                let rtype = component::ref_type(&mut self.offset, &self.bytes)?;
                let mut mode = ElementMode::Passive;
                init_arg = mode_f(&mut mode)?;
                mode.finish()?; // Does nothing
                init = ElementInit::Expressions(
                    Some(rtype),
                    ElementExpressions::new(&mut self.offset, &self.bytes)
                        .context("expressions in passive element segment")?,
                );
            }
            6 => {
                let mut mode = ElementMode::Active(
                    component::index(&mut self.offset, &self.bytes)?,
                    InstructionSequence::new(&mut self.offset, &self.bytes),
                );

                init_arg = mode_f(&mut mode)?;
                mode.finish()?;
                let rtype = component::ref_type(&mut self.offset, &self.bytes)?;
                init = ElementInit::Expressions(
                    Some(rtype),
                    ElementExpressions::new(&mut self.offset, &self.bytes)
                        .context("expressions in active element segment")?,
                );
            }
            7 => {
                let rtype = component::ref_type(&mut self.offset, &self.bytes)?;
                let mut mode = ElementMode::Declarative;
                init_arg = mode_f(&mut mode)?;
                mode.finish()?; // Does nothing
                init = ElementInit::Expressions(
                    Some(rtype),
                    ElementExpressions::new(&mut self.offset, &self.bytes)
                        .context("expressions in declarative element segment")?,
                );
            }
            _ => {
                return Err(crate::parser_bad_format_at_offset!(
                    "file" @ start,
                    "{segment_kind} is not a supported element segment mode"
                ))
            }
        }

        let result = init_f(init_arg, &mut init)?;
        init.finish()?;
        Ok(result)
    }

    /// Parses the next element segment in the section.
    pub fn parse<Y, Z, M, I>(&mut self, mode_f: M, init_f: I) -> Result<Option<Z>>
    where
        M: FnOnce(&mut ElementMode<&mut u64, &B>) -> Result<Y>,
        I: FnOnce(Y, &mut ElementInit<&mut u64, &B>) -> Result<Z>,
    {
        if self.count == 0 {
            return Ok(None);
        }

        let result = self.parse_inner(mode_f, init_f);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }

    /// Gets the expected remaining number of entires in the *element section* that have yet to be parsed.
    #[inline]
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Returns a value indicating if the *element section* is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<B: Bytes> Debug for ElemsComponent<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let bytes = &self.bytes;
        let mut elems = ElemsComponent {
            count: self.count,
            offset: self.offset,
            bytes,
        };

        let mut list = f.debug_list();

        struct Elem<'a, 'b, 'c, 'd, 'e, B: Bytes> {
            mode: ElementMode<u64, &'a B>,
            elements: &'b mut ElementInit<&'c mut u64, &'d &'e B>,
        }

        impl<B: Bytes> Debug for Elem<'_, '_, '_, '_, '_, B> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("ElementSegment")
                    .field("mode", &self.mode)
                    .field("elements", self.elements)
                    .finish()
            }
        }

        while elems.count > 0 {
            let result = elems.parse(
                |mode| {
                    Ok(match mode {
                        ElementMode::Passive => ElementMode::Passive,
                        ElementMode::Declarative => ElementMode::Declarative,
                        ElementMode::Active(table, offset) => {
                            ElementMode::Active(*table, offset.cloned())
                        }
                    })
                },
                |mode, elements| {
                    list.entry(&Elem { mode, elements });

                    Ok(())
                },
            );

            if let Err(e) = result {
                list.entry(&Result::<()>::Err(e));
            }
        }
        list.finish()
    }
}
