use crate::{
    bytes::Bytes,
    index,
    instruction_set::{self, Instruction as Instr},
    types::{self, BlockType},
    wat::{self, Writer},
};

fn write_type_use(index: index::TypeIdx, w: &mut Writer) {
    w.write_str("(type ");
    wat::write_index(false, index, w);
    w.write_char(')');
}

fn write_block_type(block_type: BlockType, w: &mut Writer) {
    match block_type {
        BlockType::Empty => (),
        BlockType::Inline(ty) => write!(w, "(result {ty})"),
        BlockType::Index(idx) => write_type_use(idx, w),
    }
}

fn write_non_zero_index<I: wat::IndexFormat>(idx: I, w: &mut Writer) {
    if Into::<u32>::into(idx) != 0 {
        w.write_char(' ');
        wat::write_index(false, idx, w);
    }
}

fn write_mem_arg(arg: &instruction_set::MemArg, w: &mut Writer) {
    write_non_zero_index(arg.memory(), w);

    if arg.offset() != 0 {
        write!(w, " offset={}", arg.offset());
    }

    if arg.align() != instruction_set::Align::None {
        write!(w, " align={}", arg.align());
    }
}

const INDENTATION: &str = "  ";

fn write_instruction<B: Bytes>(instr: &Instr<'_, B>, indentation: Option<u32>, w: &mut Writer) {
    match indentation {
        Some(level) if w.alternate() => {
            // Instruction sequence instructions start w/ nesting == 1
            for _ in 1..level {
                w.write_str(INDENTATION);
            }
        }
        _ => (),
    }

    w.write_str(instr.name());

    match instr {
        Instr::Block(ty) | Instr::Loop(ty) | Instr::If(ty) => {
            w.write_char(' ');
            write_block_type(*ty, w);
        }
        Instr::Br(target) | Instr::BrIf(target) => {
            write!(w, " {}", target.to_u32())
        }
        Instr::BrTable(entries) => {
            for target in entries.by_reference() {
                w.write_char(' ');
                wat::write_result(target.map(u32::from), w);
            }
        }
        Instr::Call(idx) | Instr::RefFunc(idx) | Instr::ReturnCall(idx) => {
            w.write_char(' ');
            wat::write_index(false, *idx, w)
        }
        Instr::CallIndirect(signature, table) | Instr::ReturnCallIndirect(signature, table) => {
            w.write_char(' ');
            wat::write_index(false, *table, w);
            write_type_use(*signature, w);
        }
        Instr::Select(types) => {
            w.write_str(" (result");
            wat::write_types(types.by_reference(), w);
            w.write_char(')');
        }
        Instr::LocalGet(idx) | Instr::LocalSet(idx) | Instr::LocalTee(idx) => {
            wat::write_index(false, *idx, w);
            w.write_char(' ');
        }
        Instr::GlobalGet(idx) | Instr::GlobalSet(idx) => {
            w.write_char(' ');
            wat::write_index(false, *idx, w);
        }
        Instr::I32Load(arg)
        | Instr::I64Load(arg)
        | Instr::F32Load(arg)
        | Instr::F64Load(arg)
        | Instr::I32Load8S(arg)
        | Instr::I32Load8U(arg)
        | Instr::I32Load16S(arg)
        | Instr::I32Load16U(arg)
        | Instr::I64Load8S(arg)
        | Instr::I64Load8U(arg)
        | Instr::I64Load16S(arg)
        | Instr::I64Load16U(arg)
        | Instr::I64Load32S(arg)
        | Instr::I64Load32U(arg)
        | Instr::I32Store(arg)
        | Instr::I64Store(arg)
        | Instr::F32Store(arg)
        | Instr::F64Store(arg)
        | Instr::I32Store8(arg)
        | Instr::I32Store16(arg)
        | Instr::I64Store8(arg)
        | Instr::I64Store16(arg)
        | Instr::I64Store32(arg)
        | Instr::V128Load(arg)
        | Instr::V128Load8x8S(arg)
        | Instr::V128Load8x8U(arg)
        | Instr::V128Load16x4S(arg)
        | Instr::V128Load16x4U(arg)
        | Instr::V128Load32x2S(arg)
        | Instr::V128Load32x2U(arg)
        | Instr::V128Load8Splat(arg)
        | Instr::V128Load16Splat(arg)
        | Instr::V128Load32Splat(arg)
        | Instr::V128Load64Splat(arg)
        | Instr::V128Load32Zero(arg)
        | Instr::V128Load64Zero(arg)
        | Instr::V128Store(arg)
        | Instr::MemoryAtomicNotify(arg)
        | Instr::MemoryAtomicWait32(arg)
        | Instr::MemoryAtomicWait64(arg)
        | Instr::I32AtomicLoad(arg)
        | Instr::I64AtomicLoad(arg)
        | Instr::I32AtomicLoad8U(arg)
        | Instr::I32AtomicLoad16U(arg)
        | Instr::I64AtomicLoad8U(arg)
        | Instr::I64AtomicLoad16U(arg)
        | Instr::I64AtomicLoad32U(arg)
        | Instr::I32AtomicStore(arg)
        | Instr::I64AtomicStore(arg)
        | Instr::I32AtomicStore8U(arg)
        | Instr::I32AtomicStore16U(arg)
        | Instr::I64AtomicStore8U(arg)
        | Instr::I64AtomicStore16U(arg)
        | Instr::I64AtomicStore32U(arg)
        | Instr::I32AtomicRmwAdd(arg)
        | Instr::I64AtomicRmwAdd(arg)
        | Instr::I32AtomicRmw8AddU(arg)
        | Instr::I32AtomicRmw16AddU(arg)
        | Instr::I64AtomicRmw8AddU(arg)
        | Instr::I64AtomicRmw16AddU(arg)
        | Instr::I64AtomicRmw32AddU(arg)
        | Instr::I32AtomicRmwSub(arg)
        | Instr::I64AtomicRmwSub(arg)
        | Instr::I32AtomicRmw8SubU(arg)
        | Instr::I32AtomicRmw16SubU(arg)
        | Instr::I64AtomicRmw8SubU(arg)
        | Instr::I64AtomicRmw16SubU(arg)
        | Instr::I64AtomicRmw32SubU(arg)
        | Instr::I32AtomicRmwAnd(arg)
        | Instr::I64AtomicRmwAnd(arg)
        | Instr::I32AtomicRmw8AndU(arg)
        | Instr::I32AtomicRmw16AndU(arg)
        | Instr::I64AtomicRmw8AndU(arg)
        | Instr::I64AtomicRmw16AndU(arg)
        | Instr::I64AtomicRmw32AndU(arg)
        | Instr::I32AtomicRmwOr(arg)
        | Instr::I64AtomicRmwOr(arg)
        | Instr::I32AtomicRmw8OrU(arg)
        | Instr::I32AtomicRmw16OrU(arg)
        | Instr::I64AtomicRmw8OrU(arg)
        | Instr::I64AtomicRmw16OrU(arg)
        | Instr::I64AtomicRmw32OrU(arg)
        | Instr::I32AtomicRmwXor(arg)
        | Instr::I64AtomicRmwXor(arg)
        | Instr::I32AtomicRmw8XorU(arg)
        | Instr::I32AtomicRmw16XorU(arg)
        | Instr::I64AtomicRmw8XorU(arg)
        | Instr::I64AtomicRmw16XorU(arg)
        | Instr::I64AtomicRmw32XorU(arg)
        | Instr::I32AtomicRmwXchg(arg)
        | Instr::I64AtomicRmwXchg(arg)
        | Instr::I32AtomicRmw8XchgU(arg)
        | Instr::I32AtomicRmw16XchgU(arg)
        | Instr::I64AtomicRmw8XchgU(arg)
        | Instr::I64AtomicRmw16XchgU(arg)
        | Instr::I64AtomicRmw32XchgU(arg)
        | Instr::I32AtomicRmwCmpxchg(arg)
        | Instr::I64AtomicRmwCmpxchg(arg)
        | Instr::I32AtomicRmw8CmpxchgU(arg)
        | Instr::I32AtomicRmw16CmpxchgU(arg)
        | Instr::I64AtomicRmw8CmpxchgU(arg)
        | Instr::I64AtomicRmw16CmpxchgU(arg)
        | Instr::I64AtomicRmw32CmpxchgU(arg) => {
            write_mem_arg(arg, w);
        }
        Instr::MemorySize(idx) | Instr::MemoryGrow(idx) | Instr::MemoryFill(idx) => {
            write_non_zero_index(*idx, w)
        }
        Instr::I32Const(i) => write!(w, " {i:#010X}"),
        Instr::I64Const(i) => write!(w, " {i:#018X}"),
        Instr::F32Const(f) => write!(w, " {:#010X}", f.to_bits()),
        Instr::F64Const(f) => write!(w, " {:#018X}", f.to_bits()),
        Instr::RefNull(rt) => w.write_str(match rt {
            types::RefType::Extern => " extern",
            types::RefType::Func => " func",
        }),
        Instr::TableGet(idx)
        | Instr::TableSet(idx)
        | Instr::TableSize(idx)
        | Instr::TableGrow(idx)
        | Instr::TableFill(idx) => {
            w.write_char(' ');
            wat::write_index(false, *idx, w)
        }
        Instr::MemoryCopy {
            destination: x,
            source: y,
        } => {
            write_non_zero_index(*x, w);
            write_non_zero_index(*y, w);
        }
        Instr::TableCopy {
            destination: x,
            source: y,
        } => {
            write_non_zero_index(*x, w);
            write_non_zero_index(*y, w);
        }
        Instr::MemoryInit(data, mem) => {
            write_non_zero_index(*mem, w);
            wat::write_index(false, *data, w);
        }
        Instr::TableInit(elem, table) => {
            write_non_zero_index(*table, w);
            wat::write_index(false, *elem, w);
        }
        Instr::DataDrop(data) => {
            wat::write_index(false, *data, w);
        }
        Instr::ElemDrop(elem) => {
            wat::write_index(false, *elem, w);
        }
        Instr::V128Load8Lane(mem, lane)
        | Instr::V128Load16Lane(mem, lane)
        | Instr::V128Load32Lane(mem, lane)
        | Instr::V128Load64Lane(mem, lane)
        | Instr::V128Store8Lane(mem, lane)
        | Instr::V128Store16Lane(mem, lane)
        | Instr::V128Store32Lane(mem, lane)
        | Instr::V128Store64Lane(mem, lane) => {
            write_mem_arg(mem, w);
            write!(w, " {lane}");
        }
        Instr::V128Const(vec) => {
            w.write_str(" i8x16");
            for b in vec.to_le_bytes() {
                write!(w, " {b:#04X}");
            }
        }
        Instr::I8x16Shuffle(lanes) => {
            for index in lanes {
                write!(w, " {index}");
            }
        }
        Instr::I8x16ExtractLaneS(lane)
        | Instr::I8x16ExtractLaneU(lane)
        | Instr::I8x16ReplaceLane(lane)
        | Instr::I16x8ExtractLaneS(lane)
        | Instr::I16x8ExtractLaneU(lane)
        | Instr::I16x8ReplaceLane(lane)
        | Instr::I32x4ExtractLane(lane)
        | Instr::I32x4ReplaceLane(lane)
        | Instr::I64x2ExtractLane(lane)
        | Instr::I64x2ReplaceLane(lane)
        | Instr::F32x4ExtractLane(lane)
        | Instr::F32x4ReplaceLane(lane)
        | Instr::F64x2ExtractLane(lane)
        | Instr::F64x2ReplaceLane(lane) => {
            write!(w, " {lane}")
        }
        _ => (),
    }

    writeln!(w);
}

impl<B: Bytes> core::fmt::Display for Instr<'_, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut w = Writer::new(f);
        write_instruction(self, None, &mut w);
        w.finish()
    }
}