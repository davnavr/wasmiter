macro_rules! opcodes {
    ($($name:ident = $value:literal,)*) => {
        /// An opcode value for a
        /// [vector instruction](https://webassembly.github.io/spec/core/binary/instructions.html#vector-instructions),
        /// which is an instruction prefixed by a
        /// [`0xFD` opcode](crate::instruction_set::Opcode::PrefixV128).
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        #[repr(u8)] // Change this to large bit width as needed
        pub enum VectorOpcode {
            $(
                #[allow(missing_docs)]
                $name = $value,
            )*
        }

        impl TryFrom<u32> for VectorOpcode {
            type Error = crate::instruction_set::InvalidPrefixedOpcode<0xFD>;

            fn try_from(opcode: u32) -> Result<Self, Self::Error> {
                match opcode {
                    $($value => Ok(Self::$name),)*
                    _ => Err(Self::Error::new(opcode)),
                }
            }
        }

        impl From<VectorOpcode> for u32 {
            #[inline]
            fn from(opcode: VectorOpcode) -> u32 {
                u32::from(opcode as u8)
            }
        }
    };
}

opcodes! {
    Load = 0,
    Load8x8S = 1,
    Load8x8U = 2,
    Load16x4S = 3,
    Load16x4U = 4,
    Load32x2S = 5,
    Load32x2U = 6,
    Load8Splat = 7,
    Load16Splat = 8,
    Load32Splat = 9,
    Load64Splat = 10,
    Store = 11,

    Const = 12,

    I8x16Shuffle = 13,

    I8x16Swizzle = 14,
    I8x16Splat = 15,
    I16x8Splat = 16,
    I32x4Splat = 17,
    I64x2Splat = 18,
    F32x4Splat = 19,
    F64x2Splat = 20,

    I8x16ExtractLaneS = 21,
    I8x16ExtractLaneU = 22,
    I8x16ReplaceLane = 23,
    I16x8ExtractLaneS = 24,
    I16x8ExtractLaneU = 25,
    I16x8ReplaceLane = 26,
    I32x4ExtractLane = 27,
    I32x4ReplaceLane = 28,
    I64x2ExtractLane = 29,
    I64x2ReplaceLane = 30,
    F32x4ExtractLane = 31,
    F32x4ReplaceLane = 32,
    F64x2ExtractLane = 33,
    F64x2ReplaceLane = 34,

    I8x16Eq = 35,
    I8x16Ne = 36,
    I8x16LtS = 37,
    I8x16LtU = 38,
    I8x16GtS = 39,
    I8x16GtU = 40,
    I8x16LeS = 41,
    I8x16LeU = 42,
    I8x16GeS = 43,
    I8x16GeU = 44,

    I16x8Eq = 45,
    I16x8Ne = 46,
    I16x8LtS = 47,
    I16x8LtU = 48,
    I16x8GtS = 49,
    I16x8GtU = 50,
    I16x8LeS = 51,
    I16x8LeU = 52,
    I16x8GeS = 53,
    I16x8GeU = 54,

    I32x4Eq = 55,
    I32x4Ne = 56,
    I32x4LtS = 57,
    I32x4LtU = 58,
    I32x4GtS = 59,
    I32x4GtU = 60,
    I32x4LeS = 61,
    I32x4LeU = 62,
    I32x4GeS = 63,
    I32x4GeU = 64,

    F32x4Eq = 65,
    F32x4Ne = 66,
    F32x4Lt = 67,
    F32x4Gt = 68,
    F32x4Le = 69,
    F32x4Ge = 70,

    F64x2Eq = 71,
    F64x2Ne = 72,
    F64x2Lt = 73,
    F64x2Gt = 74,
    F64x2Le = 75,
    F64x2Ge = 76,

    Not = 77,
    And = 78,
    AndNot = 79,
    Or = 80,
    Xor = 81,
    Bitselect = 82,
    AnyTrue = 83,

    Load8Lane = 84,
    Load16Lane = 85,
    Load32Lane = 86,
    Load64Lane = 87,
    Store8Lane = 88,
    Store16Lane = 89,
    Store32Lane = 90,
    Store64Lane = 91,
    Load32Zero = 92,
    Load64Zero = 93,

    F32x4DemoteF64x2Zero = 94,
    F64x2PromoteLowF32x4 = 95,

    I8x16Abs = 96,
    I8x16Neg = 97,
    I8x16Popcnt = 98,
    I8x16AllTrue = 99,
    I8x16Bitmask = 100,
    I8x16NarrowI16x8S = 101,
    I8x16NarrowI16x8U = 102,

    F32x4Ceil = 103,
    F32x4Floor = 104,
    F32x4Trunc = 105,
    F32x4Nearest = 106,

    I8x16Shl = 107,
    I8x16ShrS = 108,
    I8x16ShrU = 109,
    I8x16Add = 110,
    I8x16AddSatS = 111,
    I8x16AddSatU = 112,
    I8x16Sub = 113,
    I8x16SubSatS = 114,
    I8x16SubSatU = 115,

    F64x2Ceil = 116,
    F64x2Floor = 117,

    I8x16MinS = 118,
    I8x16MinU = 119,
    I8x16MaxS = 120,
    I8x16MaxU = 121,

    F64x2Trunc = 122,

    I8x16AvgrU = 123,

    I16x8ExtaddPairwiseI8x16S = 124,
    I16x8ExtaddPairwiseI8x16U = 125,
    I32x4ExtaddPairwiseI16x8S = 126,
    I32x4ExtaddPairwiseI16x8U = 127,

    I16x8Abs = 128,
    I16x8Neg = 129,
    I16x8Q15MulrSatS = 130,
    I16x8AllTrue = 131,
    I16x8Bitmask = 132,
    I16x8NarrowI32x4S = 133,
    I16x8NarrowI32x4U = 134,
    I16x8ExtendLowI8x16S = 135,
    I16x8ExtendHighI8x16S = 136,
    I16x8ExtendLowI8x16U = 137,
    I16x8ExtendHighI8x16U = 138,
    I16x8Shl = 139,
    I16x8ShrS = 140,
    I16x8ShrU = 141,
    I16x8Add = 142,
    I16x8AddSatS = 143,
    I16x8AddSatU = 144,
    I16x8Sub = 145,
    I16x8SubSatS = 146,
    I16x8SubSatU = 147,

    F64x2Nearest = 148,

    I16x8Mul = 149,
    I16x8MinS = 150,
    I16x8MinU = 151,
    I16x8MaxS = 152,
    I16x8MaxU = 153,
    I16x8AvgrU = 155,
    I16x8ExtmulLowI8x16S = 156,
    I16x8ExtmulHighI8x16S = 157,
    I16x8ExtmulLowI8x16U = 158,
    I16x8ExtmulHighI8x16U = 159,

    I32x4Abs = 160,
    I32x4Neg = 161,
    I32x4AllTrue = 163,
    I32x4Bitmask = 164,
    I32x4ExtendLowI16x8S = 167,
    I32x4ExtendHighI16x8S = 168,
    I32x4ExtendLowI16x8U = 169,
    I32x4ExtendHighI16x8U = 170,
    I32x4Shl = 171,
    I32x4ShrS = 172,
    I32x4ShrU = 173,
    I32x4Add = 174,
    I32x4Sub = 177,
    I32x4Mul = 181,
    I32x4MinS = 182,
    I32x4MinU = 183,
    I32x4MaxS = 184,
    I32x4MaxU = 185,
    I32x4DotI16x8S = 186,
    I32x4ExtmulLowI16x8S = 188,
    I32x4ExtmulHighI16x8S = 189,
    I32x4ExtmulLowI16x8U = 190,
    I32x4ExtmulHighI16x8U = 191,

    I64x2Abs = 192,
    I64x2Neg = 193,
    I64x2AllTrue = 195,
    I64x2Bitmask = 196,
    I64x2ExtendLowI32x4S = 199,
    I64x2ExtendHighI32x4S = 200,
    I64x2ExtendLowI32x4U = 201,
    I64x2ExtendHighI32x4U = 202,
    I64x2Shl = 203,
    I64x2ShrS = 204,
    I64x2ShrU = 205,
    I64x2Add = 206,
    I64x2Sub = 209,
    I64x2Mul = 213,

    I64x2Eq = 214,
    I64x2Ne = 215,
    I64x2LtS = 216,
    I64x2GtS = 217,
    I64x2LeS = 218,
    I64x2GeS = 219,

    I64x2ExtmulLowI32x4S = 220,
    I64x2ExtmulHighI32x4S = 221,
    I64x2ExtmulLowI32x4U = 222,
    I64x2ExtmulHighI32x4U = 223,

    F32x4Abs = 224,
    F32x4Neg = 225,
    F32x4Sqrt = 227,
    F32x4Add = 228,
    F32x4Sub = 229,
    F32x4Mul = 230,
    F32x4Div = 231,
    F32x4Min = 232,
    F32x4Max = 233,
    F32x4Pmin = 234,
    F32x4Pmax = 235,

    F64x2Abs = 236,
    F64x2Neg = 237,
    F64x2Sqrt = 239,
    F64x2Add = 240,
    F64x2Sub = 241,
    F64x2Mul = 242,
    F64x2Div = 243,
    F64x2Min = 244,
    F64x2Max = 245,
    F64x2Pmin = 246,
    F64x2Pmax = 247,

    I32x4TruncSatF32x4S = 248,
    I32x4TruncSatF32x4U = 249,
    F32x4ConvertI32x4S = 250,
    F32x4ConvertI32x4U = 251,
    I32x4TruncSatF64x2SZero = 252,
    I32x4TruncSatF64x2UZero = 253,
    F64x2ConvertLowI32x4S = 254,
    F64x2ConvertLowI32x4U = 255,
}
