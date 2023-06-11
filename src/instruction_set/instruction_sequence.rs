use crate::{
    bytes::Bytes,
    component, index,
    instruction_set::{
        self, FCPrefixedOpcode, FEPrefixedOpcode, Instruction, Opcode, VectorOpcode,
    },
    parser::{self, leb128, Offset, ResultExt},
};

fn memarg<B: Bytes>(offset: &mut u64, bytes: &B) -> parser::Result<instruction_set::MemArg> {
    let a = leb128::u32(offset, bytes).context("memory argument alignment")?;
    let o = leb128::u64(offset, bytes).context("memory argument offset")?;

    let (a, memory) = if a < 64 {
        (a, index::MemIdx::from(0u8))
    } else {
        (
            a - 64,
            component::index(offset, bytes).context("memory argument target")?,
        )
    };

    let align = u8::try_from(a)
        .ok()
        .and_then(instruction_set::Align::new)
        .ok_or_else(|| {
            crate::parser_bad_format!("{a} is too large to be a valid alignment power")
        })?;

    Ok(instruction_set::MemArg::new(o, align, memory))
}

/// Parses a WebAssembly [`Instruction`].
///
/// In order to ensure the instruction is completely parsed, callers should call
/// [Instruction::finish].
fn instruction<'a, 'b, B: Bytes>(
    offset: &'a mut u64,
    bytes: &'b B,
) -> parser::Result<Instruction<'a, &'b B>> {
    let opcode = Opcode::try_from(parser::one_byte_exact(offset, bytes).context("opcode byte")?)?;
    Ok(match opcode {
        Opcode::Unreachable => Instruction::Unreachable,
        Opcode::Nop => Instruction::Nop,
        Opcode::Block => Instruction::Block(component::block_type(offset, bytes)?),
        Opcode::Loop => {
            Instruction::Loop(component::block_type(offset, bytes).context("loop block type")?)
        }
        Opcode::If => {
            Instruction::If(component::block_type(offset, bytes).context("if block type")?)
        }
        Opcode::Else => Instruction::Else,
        Opcode::Try => {
            Instruction::Try(component::block_type(offset, bytes).context("try block type")?)
        }
        Opcode::Catch => Instruction::Catch(component::index(offset, bytes).context("catch tag")?),
        Opcode::Throw => Instruction::Throw(component::index(offset, bytes).context("throw tag")?),
        Opcode::Rethrow => {
            Instruction::Rethrow(component::index(offset, bytes).context("rethrow label")?)
        }
        Opcode::End => Instruction::End,
        Opcode::Br => Instruction::Br(component::index(offset, bytes).context("br label")?),
        Opcode::BrIf => Instruction::BrIf(component::index(offset, bytes).context("br_if label")?),
        Opcode::BrTable => {
            let branch_count = parser::leb128::u32(offset.offset_mut(), bytes)
                .context("could not parse branch table label count")?;

            let total_count = branch_count.checked_add(1).ok_or_else(|| {
                crate::parser_bad_format!(
                    "branch table has a label count of {branch_count}, which is too large"
                )
            })?;

            Instruction::BrTable(component::IndexVector::new(total_count, offset, bytes))
        }
        Opcode::Return => Instruction::Return,
        Opcode::Call => Instruction::Call(component::index(offset, bytes).context("call target")?),
        Opcode::CallIndirect => Instruction::CallIndirect(
            component::index(offset, bytes).context("indirect call signature")?,
            component::index(offset, bytes).context("indirect call target")?,
        ),
        Opcode::ReturnCall => {
            Instruction::ReturnCall(component::index(offset, bytes).context("tail call target")?)
        }
        Opcode::ReturnCallIndirect => Instruction::ReturnCallIndirect(
            component::index(offset, bytes).context("indirect tail call signature")?,
            component::index(offset, bytes).context("indirect tail call target")?,
        ),
        Opcode::Delegate => {
            Instruction::Delegate(component::index(offset, bytes).context("delegate label")?)
        }
        Opcode::CatchAll => Instruction::CatchAll,

        Opcode::Drop => Instruction::Drop,
        Opcode::Select => {
            Instruction::Select(component::ResultType::empty_with_offset(offset, bytes))
        }
        Opcode::SelectMany => Instruction::Select(
            component::ResultType::parse(offset, bytes).context("select types")?,
        ),

        Opcode::LocalGet => Instruction::LocalGet(component::index(offset, bytes)?),
        Opcode::LocalSet => Instruction::LocalSet(component::index(offset, bytes)?),
        Opcode::LocalTee => Instruction::LocalTee(component::index(offset, bytes)?),
        Opcode::GlobalGet => Instruction::GlobalGet(component::index(offset, bytes)?),
        Opcode::GlobalSet => Instruction::GlobalSet(component::index(offset, bytes)?),

        Opcode::TableGet => Instruction::TableGet(component::index(offset, bytes)?),
        Opcode::TableSet => Instruction::TableSet(component::index(offset, bytes)?),

        Opcode::I32Load => Instruction::I32Load(memarg(offset, bytes)?),
        Opcode::I64Load => Instruction::I64Load(memarg(offset, bytes)?),
        Opcode::F32Load => Instruction::F32Load(memarg(offset, bytes)?),
        Opcode::F64Load => Instruction::F64Load(memarg(offset, bytes)?),

        Opcode::I32Load8S => Instruction::I32Load8S(memarg(offset, bytes)?),
        Opcode::I32Load8U => Instruction::I32Load8U(memarg(offset, bytes)?),
        Opcode::I32Load16S => Instruction::I32Load16S(memarg(offset, bytes)?),
        Opcode::I32Load16U => Instruction::I32Load16U(memarg(offset, bytes)?),
        Opcode::I64Load8S => Instruction::I64Load8S(memarg(offset, bytes)?),
        Opcode::I64Load8U => Instruction::I64Load8U(memarg(offset, bytes)?),
        Opcode::I64Load16S => Instruction::I64Load16S(memarg(offset, bytes)?),
        Opcode::I64Load16U => Instruction::I64Load16U(memarg(offset, bytes)?),
        Opcode::I64Load32S => Instruction::I64Load32S(memarg(offset, bytes)?),
        Opcode::I64Load32U => Instruction::I64Load32U(memarg(offset, bytes)?),

        Opcode::I32Store => Instruction::I32Store(memarg(offset, bytes)?),
        Opcode::I64Store => Instruction::I64Store(memarg(offset, bytes)?),
        Opcode::F32Store => Instruction::F32Store(memarg(offset, bytes)?),
        Opcode::F64Store => Instruction::F64Store(memarg(offset, bytes)?),

        Opcode::I32Store8 => Instruction::I32Store8(memarg(offset, bytes)?),
        Opcode::I32Store16 => Instruction::I32Store16(memarg(offset, bytes)?),
        Opcode::I64Store8 => Instruction::I64Store8(memarg(offset, bytes)?),
        Opcode::I64Store16 => Instruction::I64Store16(memarg(offset, bytes)?),
        Opcode::I64Store32 => Instruction::I64Store32(memarg(offset, bytes)?),

        Opcode::MemorySize => Instruction::MemorySize(component::index(offset, bytes)?),
        Opcode::MemoryGrow => Instruction::MemoryGrow(component::index(offset, bytes)?),

        Opcode::I32Const => Instruction::I32Const(leb128::s32(offset, bytes)?),
        Opcode::I64Const => Instruction::I64Const(leb128::s64(offset, bytes)?),
        Opcode::F32Const => Instruction::F32Const(f32::from_le_bytes(
            parser::byte_array(offset, bytes).context("32-bit float constant")?,
        )),
        Opcode::F64Const => Instruction::F64Const(f64::from_le_bytes(
            parser::byte_array(offset, bytes).context("64-bit float constant")?,
        )),

        Opcode::I32Eqz => Instruction::I32Eqz,
        Opcode::I32Eq => Instruction::I32Eq,
        Opcode::I32Ne => Instruction::I32Ne,
        Opcode::I32LtS => Instruction::I32LtS,
        Opcode::I32LtU => Instruction::I32LtU,
        Opcode::I32GtS => Instruction::I32GtS,
        Opcode::I32GtU => Instruction::I32GtU,
        Opcode::I32LeS => Instruction::I32LeS,
        Opcode::I32LeU => Instruction::I32LeU,
        Opcode::I32GeS => Instruction::I32GeS,
        Opcode::I32GeU => Instruction::I32GeU,

        Opcode::I64Eqz => Instruction::I64Eqz,
        Opcode::I64Eq => Instruction::I64Eq,
        Opcode::I64Ne => Instruction::I64Ne,
        Opcode::I64LtS => Instruction::I64LtS,
        Opcode::I64LtU => Instruction::I64LtU,
        Opcode::I64GtS => Instruction::I64GtS,
        Opcode::I64GtU => Instruction::I64GtU,
        Opcode::I64LeS => Instruction::I64LeS,
        Opcode::I64LeU => Instruction::I64LeU,
        Opcode::I64GeS => Instruction::I64GeS,
        Opcode::I64GeU => Instruction::I64GeU,

        Opcode::F32Eq => Instruction::F32Eq,
        Opcode::F32Ne => Instruction::F32Ne,
        Opcode::F32Lt => Instruction::F32Lt,
        Opcode::F32Gt => Instruction::F32Gt,
        Opcode::F32Le => Instruction::F32Le,
        Opcode::F32Ge => Instruction::F32Ge,
        Opcode::F64Eq => Instruction::F64Eq,
        Opcode::F64Ne => Instruction::F64Ne,
        Opcode::F64Lt => Instruction::F64Lt,
        Opcode::F64Gt => Instruction::F64Gt,
        Opcode::F64Le => Instruction::F64Le,
        Opcode::F64Ge => Instruction::F64Ge,

        Opcode::I32Clz => Instruction::I32Clz,
        Opcode::I32Ctz => Instruction::I32Ctz,
        Opcode::I32Popcnt => Instruction::I32Popcnt,
        Opcode::I32Add => Instruction::I32Add,
        Opcode::I32Sub => Instruction::I32Sub,
        Opcode::I32Mul => Instruction::I32Mul,
        Opcode::I32DivS => Instruction::I32DivS,
        Opcode::I32DivU => Instruction::I32DivU,
        Opcode::I32RemS => Instruction::I32RemS,
        Opcode::I32RemU => Instruction::I32RemU,
        Opcode::I32And => Instruction::I32And,
        Opcode::I32Or => Instruction::I32Or,
        Opcode::I32Xor => Instruction::I32Xor,
        Opcode::I32Shl => Instruction::I32Shl,
        Opcode::I32ShrS => Instruction::I32ShrS,
        Opcode::I32ShrU => Instruction::I32ShrU,
        Opcode::I32Rotl => Instruction::I32Rotl,
        Opcode::I32Rotr => Instruction::I32Rotr,

        Opcode::I64Clz => Instruction::I64Clz,
        Opcode::I64Ctz => Instruction::I64Ctz,
        Opcode::I64Popcnt => Instruction::I64Popcnt,
        Opcode::I64Add => Instruction::I64Add,
        Opcode::I64Sub => Instruction::I64Sub,
        Opcode::I64Mul => Instruction::I64Mul,
        Opcode::I64DivS => Instruction::I64DivS,
        Opcode::I64DivU => Instruction::I64DivU,
        Opcode::I64RemS => Instruction::I64RemS,
        Opcode::I64RemU => Instruction::I64RemU,
        Opcode::I64And => Instruction::I64And,
        Opcode::I64Or => Instruction::I64Or,
        Opcode::I64Xor => Instruction::I64Xor,
        Opcode::I64Shl => Instruction::I64Shl,
        Opcode::I64ShrS => Instruction::I64ShrS,
        Opcode::I64ShrU => Instruction::I64ShrU,
        Opcode::I64Rotl => Instruction::I64Rotl,
        Opcode::I64Rotr => Instruction::I64Rotr,

        Opcode::F32Abs => Instruction::F32Abs,
        Opcode::F32Neg => Instruction::F32Neg,
        Opcode::F32Ceil => Instruction::F32Ceil,
        Opcode::F32Floor => Instruction::F32Floor,
        Opcode::F32Trunc => Instruction::F32Trunc,
        Opcode::F32Nearest => Instruction::F32Nearest,
        Opcode::F32Sqrt => Instruction::F32Sqrt,
        Opcode::F32Add => Instruction::F32Add,
        Opcode::F32Sub => Instruction::F32Sub,
        Opcode::F32Mul => Instruction::F32Mul,
        Opcode::F32Div => Instruction::F32Div,
        Opcode::F32Min => Instruction::F32Min,
        Opcode::F32Max => Instruction::F32Max,
        Opcode::F32Copysign => Instruction::F32Copysign,

        Opcode::F64Abs => Instruction::F64Abs,
        Opcode::F64Neg => Instruction::F64Neg,
        Opcode::F64Ceil => Instruction::F64Ceil,
        Opcode::F64Floor => Instruction::F64Floor,
        Opcode::F64Trunc => Instruction::F64Trunc,
        Opcode::F64Nearest => Instruction::F64Nearest,
        Opcode::F64Sqrt => Instruction::F64Sqrt,
        Opcode::F64Add => Instruction::F64Add,
        Opcode::F64Sub => Instruction::F64Sub,
        Opcode::F64Mul => Instruction::F64Mul,
        Opcode::F64Div => Instruction::F64Div,
        Opcode::F64Min => Instruction::F64Min,
        Opcode::F64Max => Instruction::F64Max,
        Opcode::F64Copysign => Instruction::F64Copysign,

        Opcode::I32WrapI64 => Instruction::I32WrapI64,
        Opcode::I32TruncF32S => Instruction::I32TruncF32S,
        Opcode::I32TruncF32U => Instruction::I32TruncF32U,
        Opcode::I32TruncF64S => Instruction::I32TruncF64S,
        Opcode::I32TruncF64U => Instruction::I32TruncF64U,
        Opcode::I64ExtendI32S => Instruction::I64ExtendI32S,
        Opcode::I64ExtendI32U => Instruction::I64ExtendI32U,
        Opcode::I64TruncF32S => Instruction::I64TruncF32S,
        Opcode::I64TruncF32U => Instruction::I64TruncF32U,
        Opcode::I64TruncF64S => Instruction::I64TruncF64S,
        Opcode::I64TruncF64U => Instruction::I64TruncF64U,
        Opcode::F32ConvertI32S => Instruction::F32ConvertI32S,
        Opcode::F32ConvertI32U => Instruction::F32ConvertI32U,
        Opcode::F32ConvertI64S => Instruction::F32ConvertI64S,
        Opcode::F32ConvertI64U => Instruction::F32ConvertI64U,
        Opcode::F32DemoteF64 => Instruction::F32DemoteF64,
        Opcode::F64ConvertI32S => Instruction::F64ConvertI32S,
        Opcode::F64ConvertI32U => Instruction::F64ConvertI32U,
        Opcode::F64ConvertI64S => Instruction::F64ConvertI64S,
        Opcode::F64ConvertI64U => Instruction::F64ConvertI64U,
        Opcode::F64PromoteF32 => Instruction::F64PromoteF32,
        Opcode::I32ReinterpretF32 => Instruction::I32ReinterpretF32,
        Opcode::I64ReinterpretF64 => Instruction::I64ReinterpretF64,
        Opcode::F32ReinterpretI32 => Instruction::F32ReinterpretI32,
        Opcode::F64ReinterpretI64 => Instruction::F64ReinterpretI64,

        Opcode::I32Extend8S => Instruction::I32Extend8S,
        Opcode::I32Extend16S => Instruction::I32Extend16S,
        Opcode::I64Extend8S => Instruction::I64Extend8S,
        Opcode::I64Extend16S => Instruction::I64Extend16S,
        Opcode::I64Extend32S => Instruction::I64Extend32S,

        Opcode::RefNull => {
            Instruction::RefNull(component::ref_type(offset, bytes).context("type for null")?)
        }
        Opcode::RefIsNull => Instruction::RefIsNull,
        Opcode::RefFunc => Instruction::RefFunc(
            component::index(offset, bytes).context("invalid reference to function")?,
        ),

        Opcode::PrefixFC => {
            let actual_opcode = leb128::u32(offset, bytes)
                .context("actual opcode")?
                .try_into()?;

            match actual_opcode {
                FCPrefixedOpcode::MemoryInit => Instruction::MemoryInit(
                    component::index(offset, bytes)?,
                    component::index(offset, bytes)?,
                ),
                FCPrefixedOpcode::DataDrop => {
                    Instruction::DataDrop(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::MemoryCopy => Instruction::MemoryCopy {
                    destination: component::index(offset, bytes).context("destination memory")?,
                    source: component::index(offset, bytes).context("source memory")?,
                },
                FCPrefixedOpcode::MemoryFill => {
                    Instruction::MemoryFill(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableInit => Instruction::TableInit(
                    component::index(offset, bytes)?,
                    component::index(offset, bytes)?,
                ),
                FCPrefixedOpcode::ElemDrop => {
                    Instruction::ElemDrop(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableCopy => Instruction::TableCopy {
                    destination: component::index(offset, bytes).context("destination table")?,
                    source: component::index(offset, bytes).context("source table")?,
                },
                FCPrefixedOpcode::TableGrow => {
                    Instruction::TableGrow(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableSize => {
                    Instruction::TableSize(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableFill => {
                    Instruction::TableFill(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::I32TruncSatF32S => Instruction::I32TruncSatF32S,
                FCPrefixedOpcode::I32TruncSatF32U => Instruction::I32TruncSatF32U,
                FCPrefixedOpcode::I32TruncSatF64S => Instruction::I32TruncSatF64S,
                FCPrefixedOpcode::I32TruncSatF64U => Instruction::I32TruncSatF64U,
                FCPrefixedOpcode::I64TruncSatF32S => Instruction::I64TruncSatF32S,
                FCPrefixedOpcode::I64TruncSatF32U => Instruction::I64TruncSatF32U,
                FCPrefixedOpcode::I64TruncSatF64S => Instruction::I64TruncSatF64S,
                FCPrefixedOpcode::I64TruncSatF64U => Instruction::I64TruncSatF64U,
            }
        }
        Opcode::PrefixV128 => {
            let actual_opcode = leb128::u32(offset, bytes)
                .context("actual opcode")?
                .try_into()?;

            match actual_opcode {
                VectorOpcode::Load => Instruction::V128Load(memarg(offset, bytes)?),

                VectorOpcode::Load8x8S => Instruction::V128Load8x8S(memarg(offset, bytes)?),
                VectorOpcode::Load8x8U => Instruction::V128Load8x8U(memarg(offset, bytes)?),
                VectorOpcode::Load16x4S => Instruction::V128Load16x4S(memarg(offset, bytes)?),
                VectorOpcode::Load16x4U => Instruction::V128Load16x4U(memarg(offset, bytes)?),
                VectorOpcode::Load32x2S => Instruction::V128Load32x2S(memarg(offset, bytes)?),
                VectorOpcode::Load32x2U => Instruction::V128Load32x2U(memarg(offset, bytes)?),

                VectorOpcode::Load8Splat => Instruction::V128Load8Splat(memarg(offset, bytes)?),
                VectorOpcode::Load16Splat => Instruction::V128Load16Splat(memarg(offset, bytes)?),
                VectorOpcode::Load32Splat => Instruction::V128Load32Splat(memarg(offset, bytes)?),
                VectorOpcode::Load64Splat => Instruction::V128Load64Splat(memarg(offset, bytes)?),
                VectorOpcode::Load32Zero => Instruction::V128Load32Zero(memarg(offset, bytes)?),
                VectorOpcode::Load64Zero => Instruction::V128Load64Zero(memarg(offset, bytes)?),

                VectorOpcode::Store => Instruction::V128Store(memarg(offset, bytes)?),

                VectorOpcode::Load8Lane => Instruction::V128Load8Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),
                VectorOpcode::Load16Lane => Instruction::V128Load16Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),
                VectorOpcode::Load32Lane => Instruction::V128Load32Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),
                VectorOpcode::Load64Lane => Instruction::V128Load64Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),

                VectorOpcode::Store8Lane => Instruction::V128Store8Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),
                VectorOpcode::Store16Lane => Instruction::V128Store16Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),
                VectorOpcode::Store32Lane => Instruction::V128Store32Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),
                VectorOpcode::Store64Lane => Instruction::V128Store64Lane(
                    memarg(offset, bytes)?,
                    parser::one_byte_exact(offset, bytes)?,
                ),

                VectorOpcode::Const => Instruction::V128Const(u128::from_le_bytes(
                    parser::byte_array(offset, bytes).context("constant 128-bit vector")?,
                )),

                VectorOpcode::I8x16Shuffle => Instruction::I8x16Shuffle(
                    parser::byte_array(offset, bytes).context("shuffle lane indices")?,
                ),

                VectorOpcode::I8x16ExtractLaneS => Instruction::I8x16ExtractLaneS(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::I8x16ExtractLaneU => Instruction::I8x16ExtractLaneU(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::I8x16ReplaceLane => Instruction::I8x16ReplaceLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),

                VectorOpcode::I16x8ExtractLaneS => Instruction::I16x8ExtractLaneS(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::I16x8ExtractLaneU => Instruction::I16x8ExtractLaneU(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::I16x8ReplaceLane => Instruction::I16x8ReplaceLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),

                VectorOpcode::I32x4ExtractLane => Instruction::I32x4ExtractLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::I32x4ReplaceLane => Instruction::I32x4ReplaceLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),

                VectorOpcode::I64x2ExtractLane => Instruction::I64x2ExtractLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::I64x2ReplaceLane => Instruction::I64x2ReplaceLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),

                VectorOpcode::F32x4ExtractLane => Instruction::F32x4ExtractLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::F32x4ReplaceLane => Instruction::F32x4ReplaceLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),

                VectorOpcode::F64x2ExtractLane => Instruction::F64x2ExtractLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::F64x2ReplaceLane => Instruction::F64x2ReplaceLane(
                    parser::one_byte_exact(offset, bytes).context("vector lane index")?,
                ),
                VectorOpcode::I8x16Swizzle => Instruction::I8x16Swizzle,

                VectorOpcode::I8x16Splat => Instruction::I8x16Splat,
                VectorOpcode::I16x8Splat => Instruction::I16x8Splat,
                VectorOpcode::I32x4Splat => Instruction::I32x4Splat,
                VectorOpcode::I64x2Splat => Instruction::I64x2Splat,
                VectorOpcode::F32x4Splat => Instruction::F32x4Splat,
                VectorOpcode::F64x2Splat => Instruction::F64x2Splat,

                VectorOpcode::I8x16Eq => Instruction::I8x16Eq,
                VectorOpcode::I8x16Ne => Instruction::I8x16Ne,
                VectorOpcode::I8x16LtS => Instruction::I8x16LtS,
                VectorOpcode::I8x16LtU => Instruction::I8x16LtU,
                VectorOpcode::I8x16GtS => Instruction::I8x16GtS,
                VectorOpcode::I8x16GtU => Instruction::I8x16GtU,
                VectorOpcode::I8x16LeS => Instruction::I8x16LeS,
                VectorOpcode::I8x16LeU => Instruction::I8x16LeU,
                VectorOpcode::I8x16GeS => Instruction::I8x16GeS,
                VectorOpcode::I8x16GeU => Instruction::I8x16GeU,

                VectorOpcode::I16x8Eq => Instruction::I16x8Eq,
                VectorOpcode::I16x8Ne => Instruction::I16x8Ne,
                VectorOpcode::I16x8LtS => Instruction::I16x8LtS,
                VectorOpcode::I16x8LtU => Instruction::I16x8LtU,
                VectorOpcode::I16x8GtS => Instruction::I16x8GtS,
                VectorOpcode::I16x8GtU => Instruction::I16x8GtU,
                VectorOpcode::I16x8LeS => Instruction::I16x8LeS,
                VectorOpcode::I16x8LeU => Instruction::I16x8LeU,
                VectorOpcode::I16x8GeS => Instruction::I16x8GeS,
                VectorOpcode::I16x8GeU => Instruction::I16x8GeU,

                VectorOpcode::I32x4Eq => Instruction::I32x4Eq,
                VectorOpcode::I32x4Ne => Instruction::I32x4Ne,
                VectorOpcode::I32x4LtS => Instruction::I32x4LtS,
                VectorOpcode::I32x4LtU => Instruction::I32x4LtU,
                VectorOpcode::I32x4GtS => Instruction::I32x4GtS,
                VectorOpcode::I32x4GtU => Instruction::I32x4GtU,
                VectorOpcode::I32x4LeS => Instruction::I32x4LeS,
                VectorOpcode::I32x4LeU => Instruction::I32x4LeU,
                VectorOpcode::I32x4GeS => Instruction::I32x4GeS,
                VectorOpcode::I32x4GeU => Instruction::I32x4GeU,

                VectorOpcode::I64x2Eq => Instruction::I64x2Eq,
                VectorOpcode::I64x2Ne => Instruction::I64x2Ne,
                VectorOpcode::I64x2LtS => Instruction::I64x2LtS,
                VectorOpcode::I64x2GtS => Instruction::I64x2GtS,
                VectorOpcode::I64x2LeS => Instruction::I64x2LeS,
                VectorOpcode::I64x2GeS => Instruction::I64x2GeS,

                VectorOpcode::F32x4Eq => Instruction::F32x4Eq,
                VectorOpcode::F32x4Ne => Instruction::F32x4Ne,
                VectorOpcode::F32x4Lt => Instruction::F32x4Lt,
                VectorOpcode::F32x4Gt => Instruction::F32x4Gt,
                VectorOpcode::F32x4Le => Instruction::F32x4Le,
                VectorOpcode::F32x4Ge => Instruction::F32x4Ge,

                VectorOpcode::F64x2Eq => Instruction::F64x2Eq,
                VectorOpcode::F64x2Ne => Instruction::F64x2Ne,
                VectorOpcode::F64x2Lt => Instruction::F64x2Lt,
                VectorOpcode::F64x2Gt => Instruction::F64x2Gt,
                VectorOpcode::F64x2Le => Instruction::F64x2Le,
                VectorOpcode::F64x2Ge => Instruction::F64x2Ge,

                VectorOpcode::Not => Instruction::V128Not,
                VectorOpcode::And => Instruction::V128And,
                VectorOpcode::AndNot => Instruction::V128AndNot,
                VectorOpcode::Or => Instruction::V128Or,
                VectorOpcode::Xor => Instruction::V128Xor,
                VectorOpcode::Bitselect => Instruction::V128Bitselect,
                VectorOpcode::AnyTrue => Instruction::V128AnyTrue,

                VectorOpcode::I8x16Abs => Instruction::I8x16Abs,
                VectorOpcode::I8x16Neg => Instruction::I8x16Neg,
                VectorOpcode::I8x16Popcnt => Instruction::I8x16Popcnt,
                VectorOpcode::I8x16AllTrue => Instruction::I8x16AllTrue,
                VectorOpcode::I8x16Bitmask => Instruction::I8x16Bitmask,
                VectorOpcode::I8x16NarrowI16x8S => Instruction::I8x16NarrowI16x8S,
                VectorOpcode::I8x16NarrowI16x8U => Instruction::I8x16NarrowI16x8U,
                VectorOpcode::I8x16Shl => Instruction::I8x16Shl,
                VectorOpcode::I8x16ShrS => Instruction::I8x16ShrS,
                VectorOpcode::I8x16ShrU => Instruction::I8x16ShrU,
                VectorOpcode::I8x16Add => Instruction::I8x16Add,
                VectorOpcode::I8x16AddSatS => Instruction::I8x16AddSatS,
                VectorOpcode::I8x16AddSatU => Instruction::I8x16AddSatU,
                VectorOpcode::I8x16Sub => Instruction::I8x16Sub,
                VectorOpcode::I8x16SubSatS => Instruction::I8x16SubSatS,
                VectorOpcode::I8x16SubSatU => Instruction::I8x16SubSatU,
                VectorOpcode::I8x16MinS => Instruction::I8x16MinS,
                VectorOpcode::I8x16MinU => Instruction::I8x16MinU,
                VectorOpcode::I8x16MaxS => Instruction::I8x16MaxS,
                VectorOpcode::I8x16MaxU => Instruction::I8x16MaxU,
                VectorOpcode::I8x16AvgrU => Instruction::I8x16AvgrU,

                VectorOpcode::I16x8ExtaddPairwiseI8x16S => Instruction::I16x8ExtaddPairwiseI8x16S,
                VectorOpcode::I16x8ExtaddPairwiseI8x16U => Instruction::I16x8ExtaddPairwiseI8x16U,
                VectorOpcode::I16x8Abs => Instruction::I16x8Abs,
                VectorOpcode::I16x8Neg => Instruction::I16x8Neg,
                VectorOpcode::I16x8Q15MulrSatS => Instruction::I16x8Q15MulrSatS,
                VectorOpcode::I16x8AllTrue => Instruction::I16x8AllTrue,
                VectorOpcode::I16x8Bitmask => Instruction::I16x8Bitmask,
                VectorOpcode::I16x8NarrowI32x4S => Instruction::I16x8NarrowI32x4S,
                VectorOpcode::I16x8NarrowI32x4U => Instruction::I16x8NarrowI32x4U,
                VectorOpcode::I16x8ExtendLowI8x16S => Instruction::I16x8ExtendLowI8x16S,
                VectorOpcode::I16x8ExtendHighI8x16S => Instruction::I16x8ExtendHighI8x16S,
                VectorOpcode::I16x8ExtendLowI8x16U => Instruction::I16x8ExtendLowI8x16U,
                VectorOpcode::I16x8ExtendHighI8x16U => Instruction::I16x8ExtendHighI8x16U,
                VectorOpcode::I16x8Shl => Instruction::I16x8Shl,
                VectorOpcode::I16x8ShrS => Instruction::I16x8ShrS,
                VectorOpcode::I16x8ShrU => Instruction::I16x8ShrU,
                VectorOpcode::I16x8Add => Instruction::I16x8Add,
                VectorOpcode::I16x8AddSatS => Instruction::I16x8AddSatS,
                VectorOpcode::I16x8AddSatU => Instruction::I16x8AddSatU,
                VectorOpcode::I16x8Sub => Instruction::I16x8Sub,
                VectorOpcode::I16x8SubSatS => Instruction::I16x8SubSatS,
                VectorOpcode::I16x8SubSatU => Instruction::I16x8SubSatU,
                VectorOpcode::I16x8Mul => Instruction::I16x8Mul,
                VectorOpcode::I16x8MinS => Instruction::I16x8MinS,
                VectorOpcode::I16x8MinU => Instruction::I16x8MinU,
                VectorOpcode::I16x8MaxS => Instruction::I16x8MaxS,
                VectorOpcode::I16x8MaxU => Instruction::I16x8MaxU,
                VectorOpcode::I16x8AvgrU => Instruction::I16x8AvgrU,
                VectorOpcode::I16x8ExtmulLowI8x16S => Instruction::I16x8ExtmulLowI8x16S,
                VectorOpcode::I16x8ExtmulHighI8x16S => Instruction::I16x8ExtmulHighI8x16S,
                VectorOpcode::I16x8ExtmulLowI8x16U => Instruction::I16x8ExtmulLowI8x16U,
                VectorOpcode::I16x8ExtmulHighI8x16U => Instruction::I16x8ExtmulHighI8x16U,

                VectorOpcode::I32x4ExtaddPairwiseI16x8S => Instruction::I32x4ExtaddPairwiseI16x8S,
                VectorOpcode::I32x4ExtaddPairwiseI16x8U => Instruction::I32x4ExtaddPairwiseI16x8U,
                VectorOpcode::I32x4Abs => Instruction::I32x4Abs,
                VectorOpcode::I32x4Neg => Instruction::I32x4Neg,
                VectorOpcode::I32x4AllTrue => Instruction::I32x4AllTrue,
                VectorOpcode::I32x4Bitmask => Instruction::I32x4Bitmask,
                VectorOpcode::I32x4ExtendLowI16x8S => Instruction::I32x4ExtendLowI16x8S,
                VectorOpcode::I32x4ExtendHighI16x8S => Instruction::I32x4ExtendHighI16x8S,
                VectorOpcode::I32x4ExtendLowI16x8U => Instruction::I32x4ExtendLowI16x8U,
                VectorOpcode::I32x4ExtendHighI16x8U => Instruction::I32x4ExtendHighI16x8U,
                VectorOpcode::I32x4Shl => Instruction::I32x4Shl,
                VectorOpcode::I32x4ShrS => Instruction::I32x4ShrS,
                VectorOpcode::I32x4ShrU => Instruction::I32x4ShrU,
                VectorOpcode::I32x4Add => Instruction::I32x4Add,
                VectorOpcode::I32x4Sub => Instruction::I32x4Sub,
                VectorOpcode::I32x4Mul => Instruction::I32x4Mul,
                VectorOpcode::I32x4MinS => Instruction::I32x4MinS,
                VectorOpcode::I32x4MinU => Instruction::I32x4MinU,
                VectorOpcode::I32x4MaxS => Instruction::I32x4MaxS,
                VectorOpcode::I32x4MaxU => Instruction::I32x4MaxU,
                VectorOpcode::I32x4DotI16x8S => Instruction::I32x4DotI16x8S,
                VectorOpcode::I32x4ExtmulLowI16x8S => Instruction::I32x4ExtmulLowI16x8S,
                VectorOpcode::I32x4ExtmulHighI16x8S => Instruction::I32x4ExtmulHighI16x8S,
                VectorOpcode::I32x4ExtmulLowI16x8U => Instruction::I32x4ExtmulLowI16x8U,
                VectorOpcode::I32x4ExtmulHighI16x8U => Instruction::I32x4ExtmulHighI16x8U,

                VectorOpcode::I64x2Abs => Instruction::I64x2Abs,
                VectorOpcode::I64x2Neg => Instruction::I64x2Neg,
                VectorOpcode::I64x2AllTrue => Instruction::I64x2AllTrue,
                VectorOpcode::I64x2Bitmask => Instruction::I64x2Bitmask,
                VectorOpcode::I64x2ExtendLowI32x4S => Instruction::I64x2ExtendLowI32x4S,
                VectorOpcode::I64x2ExtendHighI32x4S => Instruction::I64x2ExtendHighI32x4S,
                VectorOpcode::I64x2ExtendLowI32x4U => Instruction::I64x2ExtendLowI32x4U,
                VectorOpcode::I64x2ExtendHighI32x4U => Instruction::I64x2ExtendHighI32x4U,
                VectorOpcode::I64x2Shl => Instruction::I64x2Shl,
                VectorOpcode::I64x2ShrS => Instruction::I64x2ShrS,
                VectorOpcode::I64x2ShrU => Instruction::I64x2ShrU,
                VectorOpcode::I64x2Add => Instruction::I64x2Add,
                VectorOpcode::I64x2Sub => Instruction::I64x2Sub,
                VectorOpcode::I64x2Mul => Instruction::I64x2Mul,
                VectorOpcode::I64x2ExtmulLowI32x4S => Instruction::I64x2ExtmulLowI32x4S,
                VectorOpcode::I64x2ExtmulHighI32x4S => Instruction::I64x2ExtmulHighI32x4S,
                VectorOpcode::I64x2ExtmulLowI32x4U => Instruction::I64x2ExtmulLowI32x4U,
                VectorOpcode::I64x2ExtmulHighI32x4U => Instruction::I64x2ExtmulHighI32x4U,

                VectorOpcode::F32x4Ceil => Instruction::F32x4Ceil,
                VectorOpcode::F32x4Floor => Instruction::F32x4Floor,
                VectorOpcode::F32x4Trunc => Instruction::F32x4Trunc,
                VectorOpcode::F32x4Nearest => Instruction::F32x4Nearest,
                VectorOpcode::F32x4Abs => Instruction::F32x4Abs,
                VectorOpcode::F32x4Neg => Instruction::F32x4Neg,
                VectorOpcode::F32x4Sqrt => Instruction::F32x4Sqrt,
                VectorOpcode::F32x4Add => Instruction::F32x4Add,
                VectorOpcode::F32x4Sub => Instruction::F32x4Sub,
                VectorOpcode::F32x4Mul => Instruction::F32x4Mul,
                VectorOpcode::F32x4Div => Instruction::F32x4Div,
                VectorOpcode::F32x4Min => Instruction::F32x4Min,
                VectorOpcode::F32x4Max => Instruction::F32x4Max,
                VectorOpcode::F32x4Pmin => Instruction::F32x4Pmin,
                VectorOpcode::F32x4Pmax => Instruction::F32x4Pmax,

                VectorOpcode::F64x2Ceil => Instruction::F64x2Ceil,
                VectorOpcode::F64x2Floor => Instruction::F64x2Floor,
                VectorOpcode::F64x2Trunc => Instruction::F64x2Trunc,
                VectorOpcode::F64x2Nearest => Instruction::F64x2Nearest,
                VectorOpcode::F64x2Abs => Instruction::F64x2Abs,
                VectorOpcode::F64x2Neg => Instruction::F64x2Neg,
                VectorOpcode::F64x2Sqrt => Instruction::F64x2Sqrt,
                VectorOpcode::F64x2Add => Instruction::F64x2Add,
                VectorOpcode::F64x2Sub => Instruction::F64x2Sub,
                VectorOpcode::F64x2Mul => Instruction::F64x2Mul,
                VectorOpcode::F64x2Div => Instruction::F64x2Div,
                VectorOpcode::F64x2Min => Instruction::F64x2Min,
                VectorOpcode::F64x2Max => Instruction::F64x2Max,
                VectorOpcode::F64x2Pmin => Instruction::F64x2Pmin,
                VectorOpcode::F64x2Pmax => Instruction::F64x2Pmax,

                VectorOpcode::I32x4TruncSatF32x4S => Instruction::I32x4TruncSatF32x4S,
                VectorOpcode::I32x4TruncSatF32x4U => Instruction::I32x4TruncSatF32x4U,
                VectorOpcode::F32x4ConvertI32x4S => Instruction::F32x4ConvertI32x4S,
                VectorOpcode::F32x4ConvertI32x4U => Instruction::F32x4ConvertI32x4U,
                VectorOpcode::I32x4TruncSatF64x2SZero => Instruction::I32x4TruncSatF64x2SZero,
                VectorOpcode::I32x4TruncSatF64x2UZero => Instruction::I32x4TruncSatF64x2UZero,
                VectorOpcode::F64x2ConvertLowI32x4S => Instruction::F64x2ConvertLowI32x4S,
                VectorOpcode::F64x2ConvertLowI32x4U => Instruction::F64x2ConvertLowI32x4U,
                VectorOpcode::F32x4DemoteF64x2Zero => Instruction::F32x4DemoteF64x2Zero,
                VectorOpcode::F64x2PromoteLowF32x4 => Instruction::F64x2PromoteLowF32x4,
            }
        }
        Opcode::PrefixFE => {
            // This will eventually be a leb128::u32
            let actual_opcode =
                u32::from(parser::one_byte_exact(offset, bytes).context("actual opcode")?)
                    .try_into()?;

            match actual_opcode {
                FEPrefixedOpcode::MemoryAtomicNotify => {
                    Instruction::MemoryAtomicNotify(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::MemoryAtomicWait32 => {
                    Instruction::MemoryAtomicWait32(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::MemoryAtomicWait64 => {
                    Instruction::MemoryAtomicWait64(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicLoad => {
                    Instruction::I32AtomicLoad(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicLoad => {
                    Instruction::I64AtomicLoad(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicLoad8U => {
                    Instruction::I32AtomicLoad8U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicLoad16U => {
                    Instruction::I32AtomicLoad16U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicLoad8U => {
                    Instruction::I64AtomicLoad8U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicLoad16U => {
                    Instruction::I64AtomicLoad16U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicLoad32U => {
                    Instruction::I64AtomicLoad32U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicStore => {
                    Instruction::I32AtomicStore(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicStore => {
                    Instruction::I64AtomicStore(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicStore8U => {
                    Instruction::I32AtomicStore8U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicStore16U => {
                    Instruction::I32AtomicStore16U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicStore8U => {
                    Instruction::I64AtomicStore8U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicStore16U => {
                    Instruction::I64AtomicStore16U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicStore32U => {
                    Instruction::I64AtomicStore32U(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmwAdd => {
                    Instruction::I32AtomicRmwAdd(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmwAdd => {
                    Instruction::I64AtomicRmwAdd(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw8AddU => {
                    Instruction::I32AtomicRmw8AddU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw16AddU => {
                    Instruction::I32AtomicRmw16AddU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw8AddU => {
                    Instruction::I64AtomicRmw8AddU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw16AddU => {
                    Instruction::I64AtomicRmw16AddU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw32AddU => {
                    Instruction::I64AtomicRmw32AddU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmwSub => {
                    Instruction::I32AtomicRmwSub(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmwSub => {
                    Instruction::I64AtomicRmwSub(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw8SubU => {
                    Instruction::I32AtomicRmw8SubU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw16SubU => {
                    Instruction::I32AtomicRmw16SubU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw8SubU => {
                    Instruction::I64AtomicRmw8SubU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw16SubU => {
                    Instruction::I64AtomicRmw16SubU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw32SubU => {
                    Instruction::I64AtomicRmw32SubU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmwAnd => {
                    Instruction::I32AtomicRmwAnd(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmwAnd => {
                    Instruction::I64AtomicRmwAnd(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw8AndU => {
                    Instruction::I32AtomicRmw8AndU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw16AndU => {
                    Instruction::I32AtomicRmw16AndU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw8AndU => {
                    Instruction::I64AtomicRmw8AndU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw16AndU => {
                    Instruction::I64AtomicRmw16AndU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw32AndU => {
                    Instruction::I64AtomicRmw32AndU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmwOr => {
                    Instruction::I32AtomicRmwOr(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmwOr => {
                    Instruction::I64AtomicRmwOr(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw8OrU => {
                    Instruction::I32AtomicRmw8OrU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw16OrU => {
                    Instruction::I32AtomicRmw16OrU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw8OrU => {
                    Instruction::I64AtomicRmw8OrU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw16OrU => {
                    Instruction::I64AtomicRmw16OrU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw32OrU => {
                    Instruction::I64AtomicRmw32OrU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmwXor => {
                    Instruction::I32AtomicRmwXor(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmwXor => {
                    Instruction::I64AtomicRmwXor(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw8XorU => {
                    Instruction::I32AtomicRmw8XorU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw16XorU => {
                    Instruction::I32AtomicRmw16XorU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw8XorU => {
                    Instruction::I64AtomicRmw8XorU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw16XorU => {
                    Instruction::I64AtomicRmw16XorU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw32XorU => {
                    Instruction::I64AtomicRmw32XorU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmwXchg => {
                    Instruction::I32AtomicRmwXchg(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmwXchg => {
                    Instruction::I64AtomicRmwXchg(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw8XchgU => {
                    Instruction::I32AtomicRmw8XchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw16XchgU => {
                    Instruction::I32AtomicRmw16XchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw8XchgU => {
                    Instruction::I64AtomicRmw8XchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw16XchgU => {
                    Instruction::I64AtomicRmw16XchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw32XchgU => {
                    Instruction::I64AtomicRmw32XchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmwCmpxchg => {
                    Instruction::I32AtomicRmwCmpxchg(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmwCmpxchg => {
                    Instruction::I64AtomicRmwCmpxchg(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw8CmpxchgU => {
                    Instruction::I32AtomicRmw8CmpxchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I32AtomicRmw16CmpxchgU => {
                    Instruction::I32AtomicRmw16CmpxchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw8CmpxchgU => {
                    Instruction::I64AtomicRmw8CmpxchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw16CmpxchgU => {
                    Instruction::I64AtomicRmw16CmpxchgU(memarg(offset, bytes)?)
                }
                FEPrefixedOpcode::I64AtomicRmw32CmpxchgU => {
                    Instruction::I64AtomicRmw32CmpxchgU(memarg(offset, bytes)?)
                }
            }
        }
    }) //.context() // the opcode name
}

#[inline]
fn next_instruction<'a, T, E, B, F>(
    offset: &'a mut u64,
    bytes: &'a B,
    blocks: &mut u32,
    f: F,
) -> Result<T, E>
where
    E: From<parser::Error>,
    B: Bytes,
    F: FnOnce(&mut Instruction<'a, &'a B>) -> Result<T, E>,
{
    let mut instruction = self::instruction(offset, bytes)?;
    let result = f(&mut instruction)?;

    match instruction {
        Instruction::Block(_) | Instruction::Loop(_) | Instruction::If(_) | Instruction::Try(_) => {
            *blocks = blocks
                .checked_add(1)
                .ok_or_else(|| crate::parser_bad_format!("block nesting counter overflowed"))?;
        }
        Instruction::End => {
            // Won't underflow, check for self.blocks == 0 ensures None is returned early
            *blocks -= 1;
        }
        Instruction::Delegate(_) => {
            if *blocks > 1 {
                // Check above ensures a "delegate" won't erroneously mark the end of an expression
                *blocks -= 1;
            } else {
                return Err(crate::parser_bad_format!(
                    "expected end instruction to mark end of expression, but got delegate"
                )
                .into());
            }
        }
        _ => {}
    }

    instruction.finish()?;
    Ok(result)
}

/// Represents an expression or
/// [`expr`](https://webassembly.github.io/spec/core/syntax/instructions.html), which is a sequence
/// of instructions that is terminated by an [**end**](Instruction::End) instruction.
#[derive(Clone, Copy)]
pub struct InstructionSequence<O: Offset, B: Bytes> {
    blocks: u32,
    offset: O,
    bytes: B,
}

impl<O: Offset, B: Bytes> InstructionSequence<O, B> {
    /// Uses the given [`Bytes`] to read a sequence of instructions, starting at the given
    /// `offset`.
    pub fn new(offset: O, bytes: B) -> Self {
        Self {
            blocks: 1,
            offset,
            bytes,
        }
    }

    /// Returns a value indicating if there are more instructions remaining to be parsed.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.blocks == 0
    }

    /// Gets the number of [**block**]s that have been entered, or in other words, the remaining
    /// number of [**end**] instructions to be parsed before the instruction sequence is considered
    /// finished.
    ///
    /// Returns `0` if there are no more instructions to parse.
    ///
    /// [**block**]: https://webassembly.github.io/spec/core/syntax/instructions.html#control-instructions
    /// [**end**]: Instruction::End
    #[inline]
    pub fn nesting_level(&self) -> u32 {
        self.blocks
    }

    /// Processes the next [`Instruction`] in the sequence, providing it to the given closure.
    pub fn next<'a, T, E, F>(&'a mut self, f: F) -> Option<Result<T, E>>
    where
        E: From<parser::Error>,
        F: FnOnce(&mut Instruction<'a, &'a B>) -> Result<T, E>,
    {
        if self.is_finished() {
            return None;
        }

        let result = next_instruction(self.offset.offset_mut(), &self.bytes, &mut self.blocks, f);

        if result.is_err() {
            // If error is encountered, no more instructions should be returned
            self.blocks = 0u32;
        }

        Some(result)
    }

    /// Processes the remaining instructions in the sequence. Returns `true` if all instructions
    /// were already processed, and the offset to the byte after the last byte of the last
    /// instruction.
    ///
    /// If the expression is not terminated by an [**end**](Instruction::End) instruction, then
    /// an error is returned.
    pub fn finish(mut self) -> crate::Result<(bool, O)> {
        let mut was_finished = true;
        loop {
            match self.next(|_| crate::Result::Ok(())) {
                Some(Ok(())) => was_finished = false,
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        match self.blocks {
            0 => Ok((was_finished, self.offset)),
            1 => Err(crate::parser_bad_format!(
                "missing end instruction for expression, or blocks were not structured correctly"
            )),
            missing => Err(crate::parser_bad_format!(
                "blocks are not structured correctly, {missing} end instructions were missing"
            )),
        }
    }

    /// Clones the [`InstructionSequence`], borrowing the underlying [`Bytes`].
    #[inline]
    pub fn borrowed(&self) -> InstructionSequence<u64, &B> {
        InstructionSequence {
            blocks: self.blocks,
            offset: self.offset.offset(),
            bytes: &self.bytes,
        }
    }
}

impl<O: Offset, B: Clone + Bytes> InstructionSequence<O, &B> {
    /// Clones the [`InstructionSequence`], calling [`Clone::clone`] on the underlying [`Bytes`].
    #[inline]
    pub fn cloned(&self) -> InstructionSequence<u64, B> {
        InstructionSequence {
            blocks: self.blocks,
            offset: self.offset.offset(),
            bytes: self.bytes.clone(),
        }
    }
}

impl<O: Offset, B: Bytes> core::fmt::Debug for InstructionSequence<O, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut instructions = InstructionSequence {
            blocks: self.blocks,
            offset: self.offset.offset(),
            bytes: &self.bytes,
        };

        let mut list = f.debug_list();
        loop {
            let result = instructions.next(|i| {
                list.entry(&crate::Result::Ok(i));
                Ok(())
            });

            match result {
                None => break,
                Some(Err(e)) => {
                    list.entry(&crate::Result::<()>::Err(e));
                    break;
                }
                Some(Ok(_)) => (),
            }
        }
        list.finish()
    }
}
