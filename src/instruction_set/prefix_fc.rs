macro_rules! opcodes {
    ($($name:ident = $value:literal,)*) => {
        /// An opcode value for an instruction prefixed by a
        /// [`0xFC` opcode](crate::instruction_set::Opcode::PrefixFC).
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum FCPrefixedOpcode {
            $(
                #[allow(missing_docs)]
                $name = $value,
            )*
        }

        impl TryFrom<u32> for FCPrefixedOpcode {
            type Error = crate::instruction_set::InvalidPrefixedOpcode<0xFC>;

            fn try_from(opcode: u32) -> Result<Self, Self::Error> {
                match opcode {
                    $($value => Ok(Self::$name),)*
                    _ => Err(Self::Error::new(opcode)),
                }
            }
        }
    };
}

opcodes! {
    I32TruncSatF32S = 0,
    I32TruncSatF32U = 1,
    I32TruncSatF64S = 2,
    I32TruncSatF64U = 3,
    I64TruncSatF32S = 4,
    I64TruncSatF32U = 5,
    I64TruncSatF64S = 6,
    I64TruncSatF64U = 7,

    MemoryInit = 8,
    DataDrop = 9,
    MemoryCopy = 10,
    MemoryFill = 11,

    TableInit = 12,
    ElemDrop = 13,
    TableCopy = 14,
    TableGrow = 15,
    TableSize = 16,
    TableFill = 17,
}
