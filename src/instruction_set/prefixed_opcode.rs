macro_rules! opcodes {
    ($(
        $(#[$enum_meta:meta])*
        $enum_name:ident($prefix:literal) {
            $($name:ident = $value:literal,)*
        }
    )*) => {$(
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum $enum_name {
            $(
                #[allow(missing_docs)]
                $name = $value,
            )*
        }

        impl TryFrom<u32> for $enum_name {
            type Error = crate::instruction_set::InvalidPrefixedOpcode<$prefix>;

            fn try_from(opcode: u32) -> Result<Self, Self::Error> {
                match opcode {
                    $($value => Ok(Self::$name),)*
                    _ => Err(Self::Error::new(opcode)),
                }
            }
        }
    )*};
}

opcodes! {
    /// An opcode value for an instruction prefixed by a
    /// [`0xFC` opcode](crate::instruction_set::Opcode::PrefixFC).
    FCPrefixedOpcode(0xFC) {
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

    /// An opcode value for an instruction prefixed by a
    /// [`0xFE` opcode](crate::instruction_set::Opcode::PrefixFE).
    ///
    /// Currently, instructions in this category only include the atomic memory instructions
    /// introduced by the [WebAssembly threads proposal](https://github.com/webassembly/threads).
    FEPrefixedOpcode(0xFE) {
        MemoryAtomicNotify = 0,
        MemoryAtomicWait32 = 1,
        MemoryAtomicWait64 = 2,

        I32AtomicLoad = 0x10,
        I64AtomicLoad = 0x11,
        I32AtomicLoad8U = 0x12,
        I32AtomicLoad16U = 0x13,
        I64AtomicLoad8U = 0x14,
        I64AtomicLoad16U = 0x15,
        I64AtomicLoad32U = 0x16,

        I32AtomicStore = 0x17,
        I64AtomicStore = 0x18,
        I32AtomicStore8U = 0x19,
        I32AtomicStore16U = 0x1A,
        I64AtomicStore8U = 0x1B,
        I64AtomicStore16U = 0x1C,
        I64AtomicStore32U = 0x1D,

        I32AtomicRmwAdd = 0x1E,
        I64AtomicRmwAdd = 0x1F,
        I32AtomicRmw8AddU = 0x20,
        I32AtomicRmw16AddU = 0x21,
        I64AtomicRmw8AddU = 0x22,
        I64AtomicRmw16AddU = 0x23,
        I64AtomicRmw32AddU = 0x24,

        I32AtomicRmwSub = 0x25,
        I64AtomicRmwSub = 0x26,
        I32AtomicRmw8SubU = 0x27,
        I32AtomicRmw16SubU = 0x28,
        I64AtomicRmw8SubU = 0x29,
        I64AtomicRmw16SubU = 0x2A,
        I64AtomicRmw32SubU = 0x2B,

        I32AtomicRmwAnd = 0x2C,
        I64AtomicRmwAnd = 0x2D,
        I32AtomicRmw8AndU = 0x2E,
        I32AtomicRmw16AndU = 0x2F,
        I64AtomicRmw8AndU = 0x30,
        I64AtomicRmw16AndU = 0x31,
        I64AtomicRmw32AndU = 0x32,

        I32AtomicRmwOr = 0x33,
        I64AtomicRmwOr = 0x34,
        I32AtomicRmw8OrU = 0x35,
        I32AtomicRmw16OrU = 0x36,
        I64AtomicRmw8OrU = 0x37,
        I64AtomicRmw16OrU = 0x38,
        I64AtomicRmw32OrU = 0x39,

        I32AtomicRmwXor = 0x3A,
        I64AtomicRmwXor = 0x3B,
        I32AtomicRmw8XorU = 0x3C,
        I32AtomicRmw16XorU = 0x3D,
        I64AtomicRmw8XorU = 0x3E,
        I64AtomicRmw16XorU = 0x3F,
        I64AtomicRmw32XorU = 0x40,

        I32AtomicRmwXchg = 0x41,
        I64AtomicRmwXchg = 0x42,
        I32AtomicRmw8XchgU = 0x43,
        I32AtomicRmw16XchgU = 0x44,
        I64AtomicRmw8XchgU = 0x45,
        I64AtomicRmw16XchgU = 0x46,
        I64AtomicRmw32XchgU = 0x47,

        I32AtomicRmwCmpxchg = 0x48,
        I64AtomicRmwCmpxchg = 0x49,
        I32AtomicRmw8CmpxchgU = 0x4A,
        I32AtomicRmw16CmpxchgU = 0x4B,
        I64AtomicRmw8CmpxchgU = 0x4C,
        I64AtomicRmw16CmpxchgU = 0x4D,
        I64AtomicRmw32CmpxchgU = 0x4E,
    }
}
