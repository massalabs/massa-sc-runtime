use wasmer::wasmparser::Operator;

pub fn operator_field_str<'a>(op: &'a Operator) -> &'a str {
    match op {
        Operator::Unreachable => {
            stringify!(Unreachable)
        }
        Operator::Nop => {
            stringify!(Nop)
        }
        Operator::Block { .. } => {
            stringify!(Block)
        }
        Operator::Loop { .. } => {
            stringify!(Loop)
        }
        Operator::If { .. } => {
            stringify!(If)
        }
        Operator::Else => {
            stringify!(Else)
        }
        Operator::Try { .. } => {
            stringify!(Try)
        }
        Operator::Catch { .. } => {
            stringify!(Catch)
        }
        Operator::Throw { .. } => {
            stringify!(Throw)
        }
        Operator::Rethrow { .. } => {
            stringify!(Rethrow)
        }
        Operator::End => {
            stringify!(End)
        }
        Operator::Br { .. } => {
            stringify!(Br)
        }
        Operator::BrIf { .. } => {
            stringify!(BrIf)
        }
        Operator::BrTable { .. } => {
            stringify!(BrTable)
        }
        Operator::Return => {
            stringify!(Return)
        }
        Operator::Call { .. } => {
            stringify!(Call)
        }
        Operator::CallIndirect { .. } => {
            stringify!(CallIndirect)
        }
        Operator::ReturnCall { .. } => {
            stringify!(ReturnCall)
        }
        Operator::ReturnCallIndirect { .. } => {
            stringify!(ReturnCallIndirect)
        }
        Operator::Delegate { .. } => {
            stringify!(Delegate)
        }
        Operator::CatchAll => {
            stringify!(CatchAll)
        }
        Operator::Drop => {
            stringify!(Drop)
        }
        Operator::Select => {
            stringify!(Select)
        }
        Operator::TypedSelect { .. } => {
            stringify!(TypedSelect)
        }
        Operator::LocalGet { .. } => {
            stringify!(LocalGet)
        }
        Operator::LocalSet { .. } => {
            stringify!(LocalSet)
        }
        Operator::LocalTee { .. } => {
            stringify!(LocalTee)
        }
        Operator::GlobalGet { .. } => {
            stringify!(GlobalGet)
        }
        Operator::GlobalSet { .. } => {
            stringify!(GlobalSet)
        }
        Operator::I32Load { .. } => {
            stringify!(I32Load)
        }
        Operator::I64Load { .. } => {
            stringify!(I64Load)
        }
        Operator::F32Load { .. } => {
            stringify!(F32Load)
        }
        Operator::F64Load { .. } => {
            stringify!(F64Load)
        }
        Operator::I32Load8S { .. } => {
            stringify!(I32Load8S)
        }
        Operator::I32Load8U { .. } => {
            stringify!(I32Load8U)
        }
        Operator::I32Load16S { .. } => {
            stringify!(I32Load16S)
        }
        Operator::I32Load16U { .. } => {
            stringify!(I32Load16U)
        }
        Operator::I64Load8S { .. } => {
            stringify!(I64Load8S)
        }
        Operator::I64Load8U { .. } => {
            stringify!(I64Load8U)
        }
        Operator::I64Load16S { .. } => {
            stringify!(I64Load16S)
        }
        Operator::I64Load16U { .. } => {
            stringify!(I64Load16U)
        }
        Operator::I64Load32S { .. } => {
            stringify!(I64Load32S)
        }
        Operator::I64Load32U { .. } => {
            stringify!(I64Load32U)
        }
        Operator::I32Store { .. } => {
            stringify!(I32Store)
        }
        Operator::I64Store { .. } => {
            stringify!(I64Store)
        }
        Operator::F32Store { .. } => {
            stringify!(F32Store)
        }
        Operator::F64Store { .. } => {
            stringify!(F64Store)
        }
        Operator::I32Store8 { .. } => {
            stringify!(I32Store8)
        }
        Operator::I32Store16 { .. } => {
            stringify!(I32Store16)
        }
        Operator::I64Store8 { .. } => {
            stringify!(I64Store8)
        }
        Operator::I64Store16 { .. } => {
            stringify!(I64Store16)
        }
        Operator::I64Store32 { .. } => {
            stringify!(I64Store32)
        }
        Operator::MemorySize { .. } => {
            stringify!(MemorySize)
        }
        Operator::MemoryGrow { .. } => {
            stringify!(MemoryGrow)
        }
        Operator::I32Const { .. } => {
            stringify!(I32Const)
        }
        Operator::I64Const { .. } => {
            stringify!(I64Const)
        }
        Operator::F32Const { .. } => {
            stringify!(F32Const)
        }
        Operator::F64Const { .. } => {
            stringify!(F64Const)
        }
        Operator::RefNull { .. } => {
            stringify!(RefNull)
        }
        Operator::RefIsNull => {
            stringify!(RefIsNull)
        }
        Operator::RefFunc { .. } => {
            stringify!(RefFunc)
        }
        Operator::I32Eqz => {
            stringify!(I32Eqz)
        }
        Operator::I32Eq => {
            stringify!(I32Eq)
        }
        Operator::I32Ne => {
            stringify!(I32Ne)
        }
        Operator::I32LtS => {
            stringify!(I32LtS)
        }
        Operator::I32LtU => {
            stringify!(I32LtU)
        }
        Operator::I32GtS => {
            stringify!(I32GtS)
        }
        Operator::I32GtU => {
            stringify!(I32GtU)
        }
        Operator::I32LeS => {
            stringify!(I32LeS)
        }
        Operator::I32LeU => {
            stringify!(I32LeU)
        }
        Operator::I32GeS => {
            stringify!(I32GeS)
        }
        Operator::I32GeU => {
            stringify!(I32GeU)
        }
        Operator::I64Eqz => {
            stringify!(I64Eqz)
        }
        Operator::I64Eq => {
            stringify!(I64Eq)
        }
        Operator::I64Ne => {
            stringify!(I64Ne)
        }
        Operator::I64LtS => {
            stringify!(I64LtS)
        }
        Operator::I64LtU => {
            stringify!(I64LtU)
        }
        Operator::I64GtS => {
            stringify!(I64GtS)
        }
        Operator::I64GtU => {
            stringify!(I64GtU)
        }
        Operator::I64LeS => {
            stringify!(I64LeS)
        }
        Operator::I64LeU => {
            stringify!(I64LeU)
        }
        Operator::I64GeS => {
            stringify!(I64GeS)
        }
        Operator::I64GeU => {
            stringify!(I64GeU)
        }
        Operator::F32Eq => {
            stringify!(F32Eq)
        }
        Operator::F32Ne => {
            stringify!(F32Ne)
        }
        Operator::F32Lt => {
            stringify!(F32Lt)
        }
        Operator::F32Gt => {
            stringify!(F32Gt)
        }
        Operator::F32Le => {
            stringify!(F32Le)
        }
        Operator::F32Ge => {
            stringify!(F32Ge)
        }
        Operator::F64Eq => {
            stringify!(F64Eq)
        }
        Operator::F64Ne => {
            stringify!(F64Ne)
        }
        Operator::F64Lt => {
            stringify!(F64Lt)
        }
        Operator::F64Gt => {
            stringify!(F64Gt)
        }
        Operator::F64Le => {
            stringify!(F64Le)
        }
        Operator::F64Ge => {
            stringify!(F64Ge)
        }
        Operator::I32Clz => {
            stringify!(I32Clz)
        }
        Operator::I32Ctz => {
            stringify!(I32Ctz)
        }
        Operator::I32Popcnt => {
            stringify!(I32Popcnt)
        }
        Operator::I32Add => {
            stringify!(I32Add)
        }
        Operator::I32Sub => {
            stringify!(I32Sub)
        }
        Operator::I32Mul => {
            stringify!(I32Mul)
        }
        Operator::I32DivS => {
            stringify!(I32DivS)
        }
        Operator::I32DivU => {
            stringify!(I32DivU)
        }
        Operator::I32RemS => {
            stringify!(I32RemS)
        }
        Operator::I32RemU => {
            stringify!(I32RemU)
        }
        Operator::I32And => {
            stringify!(I32And)
        }
        Operator::I32Or => {
            stringify!(I32Or)
        }
        Operator::I32Xor => {
            stringify!(I32Xor)
        }
        Operator::I32Shl => {
            stringify!(I32Shl)
        }
        Operator::I32ShrS => {
            stringify!(I32ShrS)
        }
        Operator::I32ShrU => {
            stringify!(I32ShrU)
        }
        Operator::I32Rotl => {
            stringify!(I32Rotl)
        }
        Operator::I32Rotr => {
            stringify!(I32Rotr)
        }
        Operator::I64Clz => {
            stringify!(I64Clz)
        }
        Operator::I64Ctz => {
            stringify!(I64Ctz)
        }
        Operator::I64Popcnt => {
            stringify!(I64Popcnt)
        }
        Operator::I64Add => {
            stringify!(I64Add)
        }
        Operator::I64Sub => {
            stringify!(I64Sub)
        }
        Operator::I64Mul => {
            stringify!(I64Mul)
        }
        Operator::I64DivS => {
            stringify!(I64DivS)
        }
        Operator::I64DivU => {
            stringify!(I64DivU)
        }
        Operator::I64RemS => {
            stringify!(I64RemS)
        }
        Operator::I64RemU => {
            stringify!(I64RemU)
        }
        Operator::I64And => {
            stringify!(I64And)
        }
        Operator::I64Or => {
            stringify!(I64Or)
        }
        Operator::I64Xor => {
            stringify!(I64Xor)
        }
        Operator::I64Shl => {
            stringify!(I64Shl)
        }
        Operator::I64ShrS => {
            stringify!(I64ShrS)
        }
        Operator::I64ShrU => {
            stringify!(I64ShrU)
        }
        Operator::I64Rotl => {
            stringify!(I64Rotl)
        }
        Operator::I64Rotr => {
            stringify!(I64Rotr)
        }
        Operator::F32Abs => {
            stringify!(F32Abs)
        }
        Operator::F32Neg => {
            stringify!(F32Neg)
        }
        Operator::F32Ceil => {
            stringify!(F32Ceil)
        }
        Operator::F32Floor => {
            stringify!(F32Floor)
        }
        Operator::F32Trunc => {
            stringify!(F32Trunc)
        }
        Operator::F32Nearest => {
            stringify!(F32Nearest)
        }
        Operator::F32Sqrt => {
            stringify!(F32Sqrt)
        }
        Operator::F32Add => {
            stringify!(F32Add)
        }
        Operator::F32Sub => {
            stringify!(F32Sub)
        }
        Operator::F32Mul => {
            stringify!(F32Mul)
        }
        Operator::F32Div => {
            stringify!(F32Div)
        }
        Operator::F32Min => {
            stringify!(F32Min)
        }
        Operator::F32Max => {
            stringify!(F32Max)
        }
        Operator::F32Copysign => {
            stringify!(F32Copysign)
        }
        Operator::F64Abs => {
            stringify!(F64Abs)
        }
        Operator::F64Neg => {
            stringify!(F64Neg)
        }
        Operator::F64Ceil => {
            stringify!(F64Ceil)
        }
        Operator::F64Floor => {
            stringify!(F64Floor)
        }
        Operator::F64Trunc => {
            stringify!(F64Trunc)
        }
        Operator::F64Nearest => {
            stringify!(F64Nearest)
        }
        Operator::F64Sqrt => {
            stringify!(F64Sqrt)
        }
        Operator::F64Add => {
            stringify!(F64Add)
        }
        Operator::F64Sub => {
            stringify!(F64Sub)
        }
        Operator::F64Mul => {
            stringify!(F64Mul)
        }
        Operator::F64Div => {
            stringify!(F64Div)
        }
        Operator::F64Min => {
            stringify!(F64Min)
        }
        Operator::F64Max => {
            stringify!(F64Max)
        }
        Operator::F64Copysign => {
            stringify!(F64Copysign)
        }
        Operator::I32WrapI64 => {
            stringify!(I32WrapI64)
        }
        Operator::I32TruncF32S => {
            stringify!(I32TruncF32S)
        }
        Operator::I32TruncF32U => {
            stringify!(I32TruncF32U)
        }
        Operator::I32TruncF64S => {
            stringify!(I32TruncF64S)
        }
        Operator::I32TruncF64U => {
            stringify!(I32TruncF64U)
        }
        Operator::I64ExtendI32S => {
            stringify!(I64ExtendI32S)
        }
        Operator::I64ExtendI32U => {
            stringify!(I64ExtendI32U)
        }
        Operator::I64TruncF32S => {
            stringify!(I64TruncF32S)
        }
        Operator::I64TruncF32U => {
            stringify!(I64TruncF32U)
        }
        Operator::I64TruncF64S => {
            stringify!(I64TruncF64S)
        }
        Operator::I64TruncF64U => {
            stringify!(I64TruncF64U)
        }
        Operator::F32ConvertI32S => {
            stringify!(F32ConvertI32S)
        }
        Operator::F32ConvertI32U => {
            stringify!(F32ConvertI32U)
        }
        Operator::F32ConvertI64S => {
            stringify!(F32ConvertI64S)
        }
        Operator::F32ConvertI64U => {
            stringify!(F32ConvertI64U)
        }
        Operator::F32DemoteF64 => {
            stringify!(F32DemoteF64)
        }
        Operator::F64ConvertI32S => {
            stringify!(F64ConvertI32S)
        }
        Operator::F64ConvertI32U => {
            stringify!(F64ConvertI32U)
        }
        Operator::F64ConvertI64S => {
            stringify!(F64ConvertI64S)
        }
        Operator::F64ConvertI64U => {
            stringify!(F64ConvertI64U)
        }
        Operator::F64PromoteF32 => {
            stringify!(F64PromoteF32)
        }
        Operator::I32ReinterpretF32 => {
            stringify!(I32ReinterpretF32)
        }
        Operator::I64ReinterpretF64 => {
            stringify!(I64ReinterpretF64)
        }
        Operator::F32ReinterpretI32 => {
            stringify!(F32ReinterpretI32)
        }
        Operator::F64ReinterpretI64 => {
            stringify!(F64ReinterpretI64)
        }
        Operator::I32Extend8S => {
            stringify!(I32Extend8S)
        }
        Operator::I32Extend16S => {
            stringify!(I32Extend16S)
        }
        Operator::I64Extend8S => {
            stringify!(I64Extend8S)
        }
        Operator::I64Extend16S => {
            stringify!(I64Extend16S)
        }
        Operator::I64Extend32S => {
            stringify!(I64Extend32S)
        }
        Operator::I32TruncSatF32S => {
            stringify!(I32TruncSatF32S)
        }
        Operator::I32TruncSatF32U => {
            stringify!(I32TruncSatF32U)
        }
        Operator::I32TruncSatF64S => {
            stringify!(I32TruncSatF64S)
        }
        Operator::I32TruncSatF64U => {
            stringify!(I32TruncSatF64U)
        }
        Operator::I64TruncSatF32S => {
            stringify!(I64TruncSatF32S)
        }
        Operator::I64TruncSatF32U => {
            stringify!(I64TruncSatF32U)
        }
        Operator::I64TruncSatF64S => {
            stringify!(I64TruncSatF64S)
        }
        Operator::I64TruncSatF64U => {
            stringify!(I64TruncSatF64U)
        }
        Operator::MemoryInit { .. } => {
            stringify!(MemoryInit)
        }
        Operator::DataDrop { .. } => {
            stringify!(DataDrop)
        }
        Operator::MemoryCopy { .. } => {
            stringify!(MemoryCopy)
        }
        Operator::MemoryFill { .. } => {
            stringify!(MemoryFill)
        }
        Operator::TableInit { .. } => {
            stringify!(TableInit)
        }
        Operator::ElemDrop { .. } => {
            stringify!(ElemDrop)
        }
        Operator::TableCopy { .. } => {
            stringify!(TableCopy)
        }
        Operator::TableFill { .. } => {
            stringify!(TableFill)
        }
        Operator::TableGet { .. } => {
            stringify!(TableGet)
        }
        Operator::TableSet { .. } => {
            stringify!(TableSet)
        }
        Operator::TableGrow { .. } => {
            stringify!(TableGrow)
        }
        Operator::TableSize { .. } => {
            stringify!(TableSize)
        }
        Operator::MemoryAtomicNotify { .. } => {
            stringify!(MemoryAtomicNotify)
        }
        Operator::MemoryAtomicWait32 { .. } => {
            stringify!(MemoryAtomicWait32)
        }
        Operator::MemoryAtomicWait64 { .. } => {
            stringify!(MemoryAtomicWait64)
        }
        Operator::AtomicFence { .. } => {
            stringify!(AtomicFence)
        }
        Operator::I32AtomicLoad { .. } => {
            stringify!(I32AtomicLoad)
        }
        Operator::I64AtomicLoad { .. } => {
            stringify!(I64AtomicLoad)
        }
        Operator::I32AtomicLoad8U { .. } => {
            stringify!(I32AtomicLoad8U)
        }
        Operator::I32AtomicLoad16U { .. } => {
            stringify!(I32AtomicLoad16U)
        }
        Operator::I64AtomicLoad8U { .. } => {
            stringify!(I64AtomicLoad8U)
        }
        Operator::I64AtomicLoad16U { .. } => {
            stringify!(I64AtomicLoad16U)
        }
        Operator::I64AtomicLoad32U { .. } => {
            stringify!(I64AtomicLoad32U)
        }
        Operator::I32AtomicStore { .. } => {
            stringify!(I32AtomicStore)
        }
        Operator::I64AtomicStore { .. } => {
            stringify!(I64AtomicStore)
        }
        Operator::I32AtomicStore8 { .. } => {
            stringify!(I32AtomicStore8)
        }
        Operator::I32AtomicStore16 { .. } => {
            stringify!(I32AtomicStore16)
        }
        Operator::I64AtomicStore8 { .. } => {
            stringify!(I64AtomicStore8)
        }
        Operator::I64AtomicStore16 { .. } => {
            stringify!(I64AtomicStore16)
        }
        Operator::I64AtomicStore32 { .. } => {
            stringify!(I64AtomicStore32)
        }
        Operator::I32AtomicRmwAdd { .. } => {
            stringify!(I32AtomicRmwAdd)
        }
        Operator::I64AtomicRmwAdd { .. } => {
            stringify!(I64AtomicRmwAdd)
        }
        Operator::I32AtomicRmw8AddU { .. } => {
            stringify!(I32AtomicRmw8AddU)
        }
        Operator::I32AtomicRmw16AddU { .. } => {
            stringify!(I32AtomicRmw16AddU)
        }
        Operator::I64AtomicRmw8AddU { .. } => {
            stringify!(I64AtomicRmw8AddU)
        }
        Operator::I64AtomicRmw16AddU { .. } => {
            stringify!(I64AtomicRmw16AddU)
        }
        Operator::I64AtomicRmw32AddU { .. } => {
            stringify!(I64AtomicRmw32AddU)
        }
        Operator::I32AtomicRmwSub { .. } => {
            stringify!(I32AtomicRmwSub)
        }
        Operator::I64AtomicRmwSub { .. } => {
            stringify!(I64AtomicRmwSub)
        }
        Operator::I32AtomicRmw8SubU { .. } => {
            stringify!(I32AtomicRmw8SubU)
        }
        Operator::I32AtomicRmw16SubU { .. } => {
            stringify!(I32AtomicRmw16SubU)
        }
        Operator::I64AtomicRmw8SubU { .. } => {
            stringify!(I64AtomicRmw8SubU)
        }
        Operator::I64AtomicRmw16SubU { .. } => {
            stringify!(I64AtomicRmw16SubU)
        }
        Operator::I64AtomicRmw32SubU { .. } => {
            stringify!(I64AtomicRmw32SubU)
        }
        Operator::I32AtomicRmwAnd { .. } => {
            stringify!(I32AtomicRmwAnd)
        }
        Operator::I64AtomicRmwAnd { .. } => {
            stringify!(I64AtomicRmwAnd)
        }
        Operator::I32AtomicRmw8AndU { .. } => {
            stringify!(I32AtomicRmw8AndU)
        }
        Operator::I32AtomicRmw16AndU { .. } => {
            stringify!(I32AtomicRmw16AndU)
        }
        Operator::I64AtomicRmw8AndU { .. } => {
            stringify!(I64AtomicRmw8AndU)
        }
        Operator::I64AtomicRmw16AndU { .. } => {
            stringify!(I64AtomicRmw16AndU)
        }
        Operator::I64AtomicRmw32AndU { .. } => {
            stringify!(I64AtomicRmw32AndU)
        }
        Operator::I32AtomicRmwOr { .. } => {
            stringify!(I32AtomicRmwOr)
        }
        Operator::I64AtomicRmwOr { .. } => {
            stringify!(I64AtomicRmwOr)
        }
        Operator::I32AtomicRmw8OrU { .. } => {
            stringify!(I32AtomicRmw8OrU)
        }
        Operator::I32AtomicRmw16OrU { .. } => {
            stringify!(I32AtomicRmw16OrU)
        }
        Operator::I64AtomicRmw8OrU { .. } => {
            stringify!(I64AtomicRmw8OrU)
        }
        Operator::I64AtomicRmw16OrU { .. } => {
            stringify!(I64AtomicRmw16OrU)
        }
        Operator::I64AtomicRmw32OrU { .. } => {
            stringify!(I64AtomicRmw32OrU)
        }
        Operator::I32AtomicRmwXor { .. } => {
            stringify!(I32AtomicRmwXor)
        }
        Operator::I64AtomicRmwXor { .. } => {
            stringify!(I64AtomicRmwXor)
        }
        Operator::I32AtomicRmw8XorU { .. } => {
            stringify!(I32AtomicRmw8XorU)
        }
        Operator::I32AtomicRmw16XorU { .. } => {
            stringify!(I32AtomicRmw16XorU)
        }
        Operator::I64AtomicRmw8XorU { .. } => {
            stringify!(I64AtomicRmw8XorU)
        }
        Operator::I64AtomicRmw16XorU { .. } => {
            stringify!(I64AtomicRmw16XorU)
        }
        Operator::I64AtomicRmw32XorU { .. } => {
            stringify!(I64AtomicRmw32XorU)
        }
        Operator::I32AtomicRmwXchg { .. } => {
            stringify!(I32AtomicRmwXchg)
        }
        Operator::I64AtomicRmwXchg { .. } => {
            stringify!(I64AtomicRmwXchg)
        }
        Operator::I32AtomicRmw8XchgU { .. } => {
            stringify!(I32AtomicRmw8XchgU)
        }
        Operator::I32AtomicRmw16XchgU { .. } => {
            stringify!(I32AtomicRmw16XchgU)
        }
        Operator::I64AtomicRmw8XchgU { .. } => {
            stringify!(I64AtomicRmw8XchgU)
        }
        Operator::I64AtomicRmw16XchgU { .. } => {
            stringify!(I64AtomicRmw16XchgU)
        }
        Operator::I64AtomicRmw32XchgU { .. } => {
            stringify!(I64AtomicRmw32XchgU)
        }
        Operator::I32AtomicRmwCmpxchg { .. } => {
            stringify!(I32AtomicRmwCmpxchg)
        }
        Operator::I64AtomicRmwCmpxchg { .. } => {
            stringify!(I64AtomicRmwCmpxchg)
        }
        Operator::I32AtomicRmw8CmpxchgU { .. } => {
            stringify!(I32AtomicRmw8CmpxchgU)
        }
        Operator::I32AtomicRmw16CmpxchgU { .. } => {
            stringify!(I32AtomicRmw16CmpxchgU)
        }
        Operator::I64AtomicRmw8CmpxchgU { .. } => {
            stringify!(I64AtomicRmw8CmpxchgU)
        }
        Operator::I64AtomicRmw16CmpxchgU { .. } => {
            stringify!(I64AtomicRmw16CmpxchgU)
        }
        Operator::I64AtomicRmw32CmpxchgU { .. } => {
            stringify!(I64AtomicRmw32CmpxchgU)
        }
        Operator::V128Load { .. } => {
            stringify!(V128Load)
        }
        Operator::V128Load8x8S { .. } => {
            stringify!(V128Load8x8S)
        }
        Operator::V128Load8x8U { .. } => {
            stringify!(V128Load8x8U)
        }
        Operator::V128Load16x4S { .. } => {
            stringify!(V128Load16x4S)
        }
        Operator::V128Load16x4U { .. } => {
            stringify!(V128Load16x4U)
        }
        Operator::V128Load32x2S { .. } => {
            stringify!(V128Load32x2S)
        }
        Operator::V128Load32x2U { .. } => {
            stringify!(V128Load32x2U)
        }
        Operator::V128Load8Splat { .. } => {
            stringify!(V128Load8Splat)
        }
        Operator::V128Load16Splat { .. } => {
            stringify!(V128Load16Splat)
        }
        Operator::V128Load32Splat { .. } => {
            stringify!(V128Load32Splat)
        }
        Operator::V128Load64Splat { .. } => {
            stringify!(V128Load64Splat)
        }
        Operator::V128Load32Zero { .. } => {
            stringify!(V128Load32Zero)
        }
        Operator::V128Load64Zero { .. } => {
            stringify!(V128Load64Zero)
        }
        Operator::V128Store { .. } => {
            stringify!(V128Store)
        }
        Operator::V128Load8Lane { .. } => {
            stringify!(V128Load8Lane)
        }
        Operator::V128Load16Lane { .. } => {
            stringify!(V128Load16Lane)
        }
        Operator::V128Load32Lane { .. } => {
            stringify!(V128Load32Lane)
        }
        Operator::V128Load64Lane { .. } => {
            stringify!(V128Load64Lane)
        }
        Operator::V128Store8Lane { .. } => {
            stringify!(V128Store8Lane)
        }
        Operator::V128Store16Lane { .. } => {
            stringify!(V128Store16Lane)
        }
        Operator::V128Store32Lane { .. } => {
            stringify!(V128Store32Lane)
        }
        Operator::V128Store64Lane { .. } => {
            stringify!(V128Store64Lane)
        }
        Operator::V128Const { .. } => {
            stringify!(V128Const)
        }
        Operator::I8x16Shuffle { .. } => {
            stringify!(I8x16Shuffle)
        }
        Operator::I8x16ExtractLaneS { .. } => {
            stringify!(I8x16ExtractLaneS)
        }
        Operator::I8x16ExtractLaneU { .. } => {
            stringify!(I8x16ExtractLaneU)
        }
        Operator::I8x16ReplaceLane { .. } => {
            stringify!(I8x16ReplaceLane)
        }
        Operator::I16x8ExtractLaneS { .. } => {
            stringify!(I16x8ExtractLaneS)
        }
        Operator::I16x8ExtractLaneU { .. } => {
            stringify!(I16x8ExtractLaneU)
        }
        Operator::I16x8ReplaceLane { .. } => {
            stringify!(I16x8ReplaceLane)
        }
        Operator::I32x4ExtractLane { .. } => {
            stringify!(I32x4ExtractLane)
        }
        Operator::I32x4ReplaceLane { .. } => {
            stringify!(I32x4ReplaceLane)
        }
        Operator::I64x2ExtractLane { .. } => {
            stringify!(I64x2ExtractLane)
        }
        Operator::I64x2ReplaceLane { .. } => {
            stringify!(I64x2ReplaceLane)
        }
        Operator::F32x4ExtractLane { .. } => {
            stringify!(F32x4ExtractLane)
        }
        Operator::F32x4ReplaceLane { .. } => {
            stringify!(F32x4ReplaceLane)
        }
        Operator::F64x2ExtractLane { .. } => {
            stringify!(F64x2ExtractLane)
        }
        Operator::F64x2ReplaceLane { .. } => {
            stringify!(F64x2ReplaceLane)
        }
        Operator::I8x16Swizzle => {
            stringify!(I8x16Swizzle)
        }
        Operator::I8x16Splat => {
            stringify!(I8x16Splat)
        }
        Operator::I16x8Splat => {
            stringify!(I16x8Splat)
        }
        Operator::I32x4Splat => {
            stringify!(I32x4Splat)
        }
        Operator::I64x2Splat => {
            stringify!(I64x2Splat)
        }
        Operator::F32x4Splat => {
            stringify!(F32x4Splat)
        }
        Operator::F64x2Splat => {
            stringify!(F64x2Splat)
        }
        Operator::I8x16Eq => {
            stringify!(I8x16Eq)
        }
        Operator::I8x16Ne => {
            stringify!(I8x16Ne)
        }
        Operator::I8x16LtS => {
            stringify!(I8x16LtS)
        }
        Operator::I8x16LtU => {
            stringify!(I8x16LtU)
        }
        Operator::I8x16GtS => {
            stringify!(I8x16GtS)
        }
        Operator::I8x16GtU => {
            stringify!(I8x16GtU)
        }
        Operator::I8x16LeS => {
            stringify!(I8x16LeS)
        }
        Operator::I8x16LeU => {
            stringify!(I8x16LeU)
        }
        Operator::I8x16GeS => {
            stringify!(I8x16GeS)
        }
        Operator::I8x16GeU => {
            stringify!(I8x16GeU)
        }
        Operator::I16x8Eq => {
            stringify!(I16x8Eq)
        }
        Operator::I16x8Ne => {
            stringify!(I16x8Ne)
        }
        Operator::I16x8LtS => {
            stringify!(I16x8LtS)
        }
        Operator::I16x8LtU => {
            stringify!(I16x8LtU)
        }
        Operator::I16x8GtS => {
            stringify!(I16x8GtS)
        }
        Operator::I16x8GtU => {
            stringify!(I16x8GtU)
        }
        Operator::I16x8LeS => {
            stringify!(I16x8LeS)
        }
        Operator::I16x8LeU => {
            stringify!(I16x8LeU)
        }
        Operator::I16x8GeS => {
            stringify!(I16x8GeS)
        }
        Operator::I16x8GeU => {
            stringify!(I16x8GeU)
        }
        Operator::I32x4Eq => {
            stringify!(I32x4Eq)
        }
        Operator::I32x4Ne => {
            stringify!(I32x4Ne)
        }
        Operator::I32x4LtS => {
            stringify!(I32x4LtS)
        }
        Operator::I32x4LtU => {
            stringify!(I32x4LtU)
        }
        Operator::I32x4GtS => {
            stringify!(I32x4GtS)
        }
        Operator::I32x4GtU => {
            stringify!(I32x4GtU)
        }
        Operator::I32x4LeS => {
            stringify!(I32x4LeS)
        }
        Operator::I32x4LeU => {
            stringify!(I32x4LeU)
        }
        Operator::I32x4GeS => {
            stringify!(I32x4GeS)
        }
        Operator::I32x4GeU => {
            stringify!(I32x4GeU)
        }
        Operator::I64x2Eq => {
            stringify!(I64x2Eq)
        }
        Operator::I64x2Ne => {
            stringify!(I64x2Ne)
        }
        Operator::I64x2LtS => {
            stringify!(I64x2LtS)
        }
        Operator::I64x2GtS => {
            stringify!(I64x2GtS)
        }
        Operator::I64x2LeS => {
            stringify!(I64x2LeS)
        }
        Operator::I64x2GeS => {
            stringify!(I64x2GeS)
        }
        Operator::F32x4Eq => {
            stringify!(F32x4Eq)
        }
        Operator::F32x4Ne => {
            stringify!(F32x4Ne)
        }
        Operator::F32x4Lt => {
            stringify!(F32x4Lt)
        }
        Operator::F32x4Gt => {
            stringify!(F32x4Gt)
        }
        Operator::F32x4Le => {
            stringify!(F32x4Le)
        }
        Operator::F32x4Ge => {
            stringify!(F32x4Ge)
        }
        Operator::F64x2Eq => {
            stringify!(F64x2Eq)
        }
        Operator::F64x2Ne => {
            stringify!(F64x2Ne)
        }
        Operator::F64x2Lt => {
            stringify!(F64x2Lt)
        }
        Operator::F64x2Gt => {
            stringify!(F64x2Gt)
        }
        Operator::F64x2Le => {
            stringify!(F64x2Le)
        }
        Operator::F64x2Ge => {
            stringify!(F64x2Ge)
        }
        Operator::V128Not => {
            stringify!(V128Not)
        }
        Operator::V128And => {
            stringify!(V128And)
        }
        Operator::V128AndNot => {
            stringify!(V128AndNot)
        }
        Operator::V128Or => {
            stringify!(V128Or)
        }
        Operator::V128Xor => {
            stringify!(V128Xor)
        }
        Operator::V128Bitselect => {
            stringify!(V128Bitselect)
        }
        Operator::V128AnyTrue => {
            stringify!(V128AnyTrue)
        }
        Operator::I8x16Abs => {
            stringify!(I8x16Abs)
        }
        Operator::I8x16Neg => {
            stringify!(I8x16Neg)
        }
        Operator::I8x16Popcnt => {
            stringify!(I8x16Popcnt)
        }
        Operator::I8x16AllTrue => {
            stringify!(I8x16AllTrue)
        }
        Operator::I8x16Bitmask => {
            stringify!(I8x16Bitmask)
        }
        Operator::I8x16NarrowI16x8S => {
            stringify!(I8x16NarrowI16x8S)
        }
        Operator::I8x16NarrowI16x8U => {
            stringify!(I8x16NarrowI16x8U)
        }
        Operator::I8x16Shl => {
            stringify!(I8x16Shl)
        }
        Operator::I8x16ShrS => {
            stringify!(I8x16ShrS)
        }
        Operator::I8x16ShrU => {
            stringify!(I8x16ShrU)
        }
        Operator::I8x16Add => {
            stringify!(I8x16Add)
        }
        Operator::I8x16AddSatS => {
            stringify!(I8x16AddSatS)
        }
        Operator::I8x16AddSatU => {
            stringify!(I8x16AddSatU)
        }
        Operator::I8x16Sub => {
            stringify!(I8x16Sub)
        }
        Operator::I8x16SubSatS => {
            stringify!(I8x16SubSatS)
        }
        Operator::I8x16SubSatU => {
            stringify!(I8x16SubSatU)
        }
        Operator::I8x16MinS => {
            stringify!(I8x16MinS)
        }
        Operator::I8x16MinU => {
            stringify!(I8x16MinU)
        }
        Operator::I8x16MaxS => {
            stringify!(I8x16MaxS)
        }
        Operator::I8x16MaxU => {
            stringify!(I8x16MaxU)
        }
        Operator::I16x8ExtAddPairwiseI8x16S => {
            stringify!(I16x8ExtAddPairwiseI8x16S)
        }
        Operator::I16x8ExtAddPairwiseI8x16U => {
            stringify!(I16x8ExtAddPairwiseI8x16U)
        }
        Operator::I16x8Abs => {
            stringify!(I16x8Abs)
        }
        Operator::I16x8Neg => {
            stringify!(I16x8Neg)
        }
        Operator::I16x8Q15MulrSatS => {
            stringify!(I16x8Q15MulrSatS)
        }
        Operator::I16x8AllTrue => {
            stringify!(I16x8AllTrue)
        }
        Operator::I16x8Bitmask => {
            stringify!(I16x8Bitmask)
        }
        Operator::I16x8NarrowI32x4S => {
            stringify!(I16x8NarrowI32x4S)
        }
        Operator::I16x8NarrowI32x4U => {
            stringify!(I16x8NarrowI32x4U)
        }
        Operator::I16x8ExtendLowI8x16S => {
            stringify!(I16x8ExtendLowI8x16S)
        }
        Operator::I16x8ExtendHighI8x16S => {
            stringify!(I16x8ExtendHighI8x16S)
        }
        Operator::I16x8ExtendLowI8x16U => {
            stringify!(I16x8ExtendLowI8x16U)
        }
        Operator::I16x8ExtendHighI8x16U => {
            stringify!(I16x8ExtendHighI8x16U)
        }
        Operator::I16x8Shl => {
            stringify!(I16x8Shl)
        }
        Operator::I16x8ShrS => {
            stringify!(I16x8ShrS)
        }
        Operator::I16x8ShrU => {
            stringify!(I16x8ShrU)
        }
        Operator::I16x8Add => {
            stringify!(I16x8Add)
        }
        Operator::I16x8AddSatS => {
            stringify!(I16x8AddSatS)
        }
        Operator::I16x8AddSatU => {
            stringify!(I16x8AddSatU)
        }
        Operator::I16x8Sub => {
            stringify!(I16x8Sub)
        }
        Operator::I16x8SubSatS => {
            stringify!(I16x8SubSatS)
        }
        Operator::I16x8SubSatU => {
            stringify!(I16x8SubSatU)
        }
        Operator::I16x8Mul => {
            stringify!(I16x8Mul)
        }
        Operator::I16x8MinS => {
            stringify!(I16x8MinS)
        }
        Operator::I16x8MinU => {
            stringify!(I16x8MinU)
        }
        Operator::I16x8MaxS => {
            stringify!(I16x8MaxS)
        }
        Operator::I16x8MaxU => {
            stringify!(I16x8MaxU)
        }
        Operator::I16x8ExtMulLowI8x16S => {
            stringify!(I16x8ExtMulLowI8x16S)
        }
        Operator::I16x8ExtMulHighI8x16S => {
            stringify!(I16x8ExtMulHighI8x16S)
        }
        Operator::I16x8ExtMulLowI8x16U => {
            stringify!(I16x8ExtMulLowI8x16U)
        }
        Operator::I16x8ExtMulHighI8x16U => {
            stringify!(I16x8ExtMulHighI8x16U)
        }
        Operator::I32x4ExtAddPairwiseI16x8S => {
            stringify!(I32x4ExtAddPairwiseI16x8S)
        }
        Operator::I32x4ExtAddPairwiseI16x8U => {
            stringify!(I32x4ExtAddPairwiseI16x8U)
        }
        Operator::I32x4Abs => {
            stringify!(I32x4Abs)
        }
        Operator::I32x4Neg => {
            stringify!(I32x4Neg)
        }
        Operator::I32x4AllTrue => {
            stringify!(I32x4AllTrue)
        }
        Operator::I32x4Bitmask => {
            stringify!(I32x4Bitmask)
        }
        Operator::I32x4ExtendLowI16x8S => {
            stringify!(I32x4ExtendLowI16x8S)
        }
        Operator::I32x4ExtendHighI16x8S => {
            stringify!(I32x4ExtendHighI16x8S)
        }
        Operator::I32x4ExtendLowI16x8U => {
            stringify!(I32x4ExtendLowI16x8U)
        }
        Operator::I32x4ExtendHighI16x8U => {
            stringify!(I32x4ExtendHighI16x8U)
        }
        Operator::I32x4Shl => {
            stringify!(I32x4Shl)
        }
        Operator::I32x4ShrS => {
            stringify!(I32x4ShrS)
        }
        Operator::I32x4ShrU => {
            stringify!(I32x4ShrU)
        }
        Operator::I32x4Add => {
            stringify!(I32x4Add)
        }
        Operator::I32x4Sub => {
            stringify!(I32x4Sub)
        }
        Operator::I32x4Mul => {
            stringify!(I32x4Mul)
        }
        Operator::I32x4MinS => {
            stringify!(I32x4MinS)
        }
        Operator::I32x4MinU => {
            stringify!(I32x4MinU)
        }
        Operator::I32x4MaxS => {
            stringify!(I32x4MaxS)
        }
        Operator::I32x4MaxU => {
            stringify!(I32x4MaxU)
        }
        Operator::I32x4DotI16x8S => {
            stringify!(I32x4DotI16x8S)
        }
        Operator::I32x4ExtMulLowI16x8S => {
            stringify!(I32x4ExtMulLowI16x8S)
        }
        Operator::I32x4ExtMulHighI16x8S => {
            stringify!(I32x4ExtMulHighI16x8S)
        }
        Operator::I32x4ExtMulLowI16x8U => {
            stringify!(I32x4ExtMulLowI16x8U)
        }
        Operator::I32x4ExtMulHighI16x8U => {
            stringify!(I32x4ExtMulHighI16x8U)
        }
        Operator::I64x2Abs => {
            stringify!(I64x2Abs)
        }
        Operator::I64x2Neg => {
            stringify!(I64x2Neg)
        }
        Operator::I64x2AllTrue => {
            stringify!(I64x2AllTrue)
        }
        Operator::I64x2Bitmask => {
            stringify!(I64x2Bitmask)
        }
        Operator::I64x2ExtendLowI32x4S => {
            stringify!(I64x2ExtendLowI32x4S)
        }
        Operator::I64x2ExtendHighI32x4S => {
            stringify!(I64x2ExtendHighI32x4S)
        }
        Operator::I64x2ExtendLowI32x4U => {
            stringify!(I64x2ExtendLowI32x4U)
        }
        Operator::I64x2ExtendHighI32x4U => {
            stringify!(I64x2ExtendHighI32x4U)
        }
        Operator::I64x2Shl => {
            stringify!(I64x2Shl)
        }
        Operator::I64x2ShrS => {
            stringify!(I64x2ShrS)
        }
        Operator::I64x2ShrU => {
            stringify!(I64x2ShrU)
        }
        Operator::I64x2Add => {
            stringify!(I64x2Add)
        }
        Operator::I64x2Sub => {
            stringify!(I64x2Sub)
        }
        Operator::I64x2Mul => {
            stringify!(I64x2Mul)
        }
        Operator::I64x2ExtMulLowI32x4S => {
            stringify!(I64x2ExtMulLowI32x4S)
        }
        Operator::I64x2ExtMulHighI32x4S => {
            stringify!(I64x2ExtMulHighI32x4S)
        }
        Operator::I64x2ExtMulLowI32x4U => {
            stringify!(I64x2ExtMulLowI32x4U)
        }
        Operator::I64x2ExtMulHighI32x4U => {
            stringify!(I64x2ExtMulHighI32x4U)
        }
        Operator::F32x4Ceil => {
            stringify!(F32x4Ceil)
        }
        Operator::F32x4Floor => {
            stringify!(F32x4Floor)
        }
        Operator::F32x4Trunc => {
            stringify!(F32x4Trunc)
        }
        Operator::F32x4Nearest => {
            stringify!(F32x4Nearest)
        }
        Operator::F32x4Abs => {
            stringify!(F32x4Abs)
        }
        Operator::F32x4Neg => {
            stringify!(F32x4Neg)
        }
        Operator::F32x4Sqrt => {
            stringify!(F32x4Sqrt)
        }
        Operator::F32x4Add => {
            stringify!(F32x4Add)
        }
        Operator::F32x4Sub => {
            stringify!(F32x4Sub)
        }
        Operator::F32x4Mul => {
            stringify!(F32x4Mul)
        }
        Operator::F32x4Div => {
            stringify!(F32x4Div)
        }
        Operator::F32x4Min => {
            stringify!(F32x4Min)
        }
        Operator::F32x4Max => {
            stringify!(F32x4Max)
        }
        Operator::F32x4PMin => {
            stringify!(F32x4PMin)
        }
        Operator::F32x4PMax => {
            stringify!(F32x4PMax)
        }
        Operator::F64x2Ceil => {
            stringify!(F64x2Ceil)
        }
        Operator::F64x2Floor => {
            stringify!(F64x2Floor)
        }
        Operator::F64x2Trunc => {
            stringify!(F64x2Trunc)
        }
        Operator::F64x2Nearest => {
            stringify!(F64x2Nearest)
        }
        Operator::F64x2Abs => {
            stringify!(F64x2Abs)
        }
        Operator::F64x2Neg => {
            stringify!(F64x2Neg)
        }
        Operator::F64x2Sqrt => {
            stringify!(F64x2Sqrt)
        }
        Operator::F64x2Add => {
            stringify!(F64x2Add)
        }
        Operator::F64x2Sub => {
            stringify!(F64x2Sub)
        }
        Operator::F64x2Mul => {
            stringify!(F64x2Mul)
        }
        Operator::F64x2Div => {
            stringify!(F64x2Div)
        }
        Operator::F64x2Min => {
            stringify!(F64x2Min)
        }
        Operator::F64x2Max => {
            stringify!(F64x2Max)
        }
        Operator::F64x2PMin => {
            stringify!(F64x2PMin)
        }
        Operator::F64x2PMax => {
            stringify!(F64x2PMax)
        }
        Operator::I32x4TruncSatF32x4S => {
            stringify!(I32x4TruncSatF32x4S)
        }
        Operator::I32x4TruncSatF32x4U => {
            stringify!(I32x4TruncSatF32x4U)
        }
        Operator::F32x4ConvertI32x4S => {
            stringify!(F32x4ConvertI32x4S)
        }
        Operator::F32x4ConvertI32x4U => {
            stringify!(F32x4ConvertI32x4U)
        }
        Operator::I32x4TruncSatF64x2SZero => {
            stringify!(I32x4TruncSatF64x2SZero)
        }
        Operator::I32x4TruncSatF64x2UZero => {
            stringify!(I32x4TruncSatF64x2UZero)
        }
        Operator::F64x2ConvertLowI32x4S => {
            stringify!(F64x2ConvertLowI32x4S)
        }
        Operator::F64x2ConvertLowI32x4U => {
            stringify!(F64x2ConvertLowI32x4U)
        }
        Operator::F32x4DemoteF64x2Zero => {
            stringify!(F32x4DemoteF64x2Zero)
        }
        Operator::F64x2PromoteLowF32x4 => {
            stringify!(F64x2PromoteLowF32x4)
        }
        Operator::I8x16RelaxedSwizzle => {
            stringify!(I8x16RelaxedSwizzle)
        }
        Operator::F32x4RelaxedMin => {
            stringify!(F32x4RelaxedMin)
        }
        Operator::F32x4RelaxedMax => {
            stringify!(F32x4RelaxedMax)
        }
        Operator::F64x2RelaxedMin => {
            stringify!(F64x2RelaxedMin)
        }
        Operator::F64x2RelaxedMax => {
            stringify!(F64x2RelaxedMax)
        }
        Operator::I8x16AvgrU => stringify!(I8x16AvgrU),
        Operator::I16x8AvgrU => stringify!(I16x8AvgrU),
        Operator::I8x16RelaxedLaneselect => stringify!(I8x16RelaxedLaneselect),
        Operator::I16x8RelaxedLaneselect => stringify!(I16x8RelaxedLaneselect),
        Operator::I32x4RelaxedLaneselect => stringify!(I32x4RelaxedLaneselect),
        Operator::I64x2RelaxedLaneselect => stringify!(I64x2RelaxedLaneselect),
        Operator::I16x8RelaxedQ15mulrS => stringify!(I16x8RelaxedQ15mulrS),
        Operator::TryTable { .. } => stringify!(TryTable),
        Operator::ThrowRef => stringify!(ThrowRef),
        Operator::RefEq => stringify!(RefEq),
        Operator::StructNew { .. } => stringify!(StructNew),
        Operator::StructNewDefault { .. } => stringify!(StructNewDefault),
        Operator::StructGet { .. } => stringify!(StructGet),
        Operator::StructGetS { .. } => stringify!(StructGetS),
        Operator::StructGetU { .. } => stringify!(StructGetU),
        Operator::StructSet { .. } => stringify!(StructSet),
        Operator::ArrayNew { .. } => stringify!(ArrayNew),
        Operator::ArrayNewDefault { .. } => stringify!(ArrayNewDefault),
        Operator::ArrayNewFixed { .. } => stringify!(ArrayNewFixed),
        Operator::ArrayNewData { .. } => stringify!(ArrayNewData),
        Operator::ArrayNewElem { .. } => stringify!(ArrayNewElem),
        Operator::ArrayGet { .. } => stringify!(ArrayGet),
        Operator::ArrayGetS { .. } => stringify!(ArrayGetS),
        Operator::ArrayGetU { .. } => stringify!(ArrayGetU),
        Operator::ArraySet { .. } => stringify!(ArraySet),
        Operator::ArrayLen => stringify!(ArrayLen),
        Operator::ArrayFill { .. } => stringify!(ArrayFill),
        Operator::ArrayCopy { .. } => stringify!(ArrayCopy),
        Operator::ArrayInitData { .. } => stringify!(ArrayInitData),
        Operator::ArrayInitElem { .. } => stringify!(ArrayInitElem),
        Operator::RefTestNonNull { .. } => stringify!(RefTestNonNull),
        Operator::RefTestNullable { .. } => stringify!(RefTestNullable),
        Operator::RefCastNonNull { .. } => stringify!(RefCastNonNull),
        Operator::RefCastNullable { .. } => stringify!(RefCastNullable),
        Operator::BrOnCast { .. } => stringify!(BrOnCast),
        Operator::BrOnCastFail { .. } => stringify!(BrOnCastFail),
        Operator::AnyConvertExtern => stringify!(AnyConvertExtern),
        Operator::ExternConvertAny => stringify!(ExternConvertAny),
        Operator::RefI31 => stringify!(RefI31),
        Operator::I31GetS => stringify!(I31GetS),
        Operator::I31GetU => stringify!(I31GetU),
        Operator::MemoryDiscard { .. } => stringify!(MemoryDiscard),
        Operator::I32x4RelaxedTruncF32x4S => stringify!(I32x4RelaxedTruncF32x4S),
        Operator::I32x4RelaxedTruncF32x4U => stringify!(I32x4RelaxedTruncF32x4U),
        Operator::I32x4RelaxedTruncF64x2SZero => stringify!(I32x4RelaxedTruncF64x2SZero),
        Operator::I32x4RelaxedTruncF64x2UZero => stringify!(I32x4RelaxedTruncF64x2UZero),
        Operator::F32x4RelaxedMadd => stringify!(F32x4RelaxedMadd),
        Operator::F32x4RelaxedNmadd => stringify!(F32x4RelaxedNmadd),
        Operator::F64x2RelaxedMadd => stringify!(F64x2RelaxedMadd),
        Operator::F64x2RelaxedNmadd => stringify!(F64x2RelaxedNmadd),
        Operator::I16x8RelaxedDotI8x16I7x16S => stringify!(I16x8RelaxedDotI8x16I7x16S),
        Operator::I32x4RelaxedDotI8x16I7x16AddS => stringify!(I32x4RelaxedDotI8x16I7x16AddS),
        Operator::CallRef { .. } => stringify!(CallRef),
        Operator::ReturnCallRef { .. } => stringify!(ReturnCallRef),
        Operator::RefAsNonNull => stringify!(RefAsNonNull),
        Operator::BrOnNull { .. } => stringify!(BrOnNull),
        Operator::BrOnNonNull { .. } => stringify!(BrOnNonNull),
    }
}

pub(crate) const OPERATOR_VARIANTS: [&str; 529] = [
    "Unreachable",
    "Nop",
    "Block",
    "Loop",
    "If",
    "Else",
    "Try",
    "Catch",
    "Throw",
    "Rethrow",
    "End",
    "Br",
    "BrIf",
    "BrTable",
    "Return",
    "Call",
    "CallIndirect",
    "ReturnCall",
    "ReturnCallIndirect",
    "Delegate",
    "CatchAll",
    "Drop",
    "Select",
    "TypedSelect",
    "LocalGet",
    "LocalSet",
    "LocalTee",
    "GlobalGet",
    "GlobalSet",
    "I32Load",
    "I64Load",
    "F32Load",
    "F64Load",
    "I32Load8S",
    "I32Load8U",
    "I32Load16S",
    "I32Load16U",
    "I64Load8S",
    "I64Load8U",
    "I64Load16S",
    "I64Load16U",
    "I64Load32S",
    "I64Load32U",
    "I32Store",
    "I64Store",
    "F32Store",
    "F64Store",
    "I32Store8",
    "I32Store16",
    "I64Store8",
    "I64Store16",
    "I64Store32",
    "MemorySize",
    "MemoryGrow",
    "I32Const",
    "I64Const",
    "F32Const",
    "F64Const",
    "RefNull",
    "RefIsNull",
    "RefFunc",
    "I32Eqz",
    "I32Eq",
    "I32Ne",
    "I32LtS",
    "I32LtU",
    "I32GtS",
    "I32GtU",
    "I32LeS",
    "I32LeU",
    "I32GeS",
    "I32GeU",
    "I64Eqz",
    "I64Eq",
    "I64Ne",
    "I64LtS",
    "I64LtU",
    "I64GtS",
    "I64GtU",
    "I64LeS",
    "I64LeU",
    "I64GeS",
    "I64GeU",
    "F32Eq",
    "F32Ne",
    "F32Lt",
    "F32Gt",
    "F32Le",
    "F32Ge",
    "F64Eq",
    "F64Ne",
    "F64Lt",
    "F64Gt",
    "F64Le",
    "F64Ge",
    "I32Clz",
    "I32Ctz",
    "I32Popcnt",
    "I32Add",
    "I32Sub",
    "I32Mul",
    "I32DivS",
    "I32DivU",
    "I32RemS",
    "I32RemU",
    "I32And",
    "I32Or",
    "I32Xor",
    "I32Shl",
    "I32ShrS",
    "I32ShrU",
    "I32Rotl",
    "I32Rotr",
    "I64Clz",
    "I64Ctz",
    "I64Popcnt",
    "I64Add",
    "I64Sub",
    "I64Mul",
    "I64DivS",
    "I64DivU",
    "I64RemS",
    "I64RemU",
    "I64And",
    "I64Or",
    "I64Xor",
    "I64Shl",
    "I64ShrS",
    "I64ShrU",
    "I64Rotl",
    "I64Rotr",
    "F32Abs",
    "F32Neg",
    "F32Ceil",
    "F32Floor",
    "F32Trunc",
    "F32Nearest",
    "F32Sqrt",
    "F32Add",
    "F32Sub",
    "F32Mul",
    "F32Div",
    "F32Min",
    "F32Max",
    "F32Copysign",
    "F64Abs",
    "F64Neg",
    "F64Ceil",
    "F64Floor",
    "F64Trunc",
    "F64Nearest",
    "F64Sqrt",
    "F64Add",
    "F64Sub",
    "F64Mul",
    "F64Div",
    "F64Min",
    "F64Max",
    "F64Copysign",
    "I32WrapI64",
    "I32TruncF32S",
    "I32TruncF32U",
    "I32TruncF64S",
    "I32TruncF64U",
    "I64ExtendI32S",
    "I64ExtendI32U",
    "I64TruncF32S",
    "I64TruncF32U",
    "I64TruncF64S",
    "I64TruncF64U",
    "F32ConvertI32S",
    "F32ConvertI32U",
    "F32ConvertI64S",
    "F32ConvertI64U",
    "F32DemoteF64",
    "F64ConvertI32S",
    "F64ConvertI32U",
    "F64ConvertI64S",
    "F64ConvertI64U",
    "F64PromoteF32",
    "I32ReinterpretF32",
    "I64ReinterpretF64",
    "F32ReinterpretI32",
    "F64ReinterpretI64",
    "I32Extend8S",
    "I32Extend16S",
    "I64Extend8S",
    "I64Extend16S",
    "I64Extend32S",
    "I32TruncSatF32S",
    "I32TruncSatF32U",
    "I32TruncSatF64S",
    "I32TruncSatF64U",
    "I64TruncSatF32S",
    "I64TruncSatF32U",
    "I64TruncSatF64S",
    "I64TruncSatF64U",
    "MemoryInit",
    "DataDrop",
    "MemoryCopy",
    "MemoryFill",
    "TableInit",
    "ElemDrop",
    "TableCopy",
    "TableFill",
    "TableGet",
    "TableSet",
    "TableGrow",
    "TableSize",
    "MemoryAtomicNotify",
    "MemoryAtomicWait32",
    "MemoryAtomicWait64",
    "AtomicFence",
    "I32AtomicLoad",
    "I64AtomicLoad",
    "I32AtomicLoad8U",
    "I32AtomicLoad16U",
    "I64AtomicLoad8U",
    "I64AtomicLoad16U",
    "I64AtomicLoad32U",
    "I32AtomicStore",
    "I64AtomicStore",
    "I32AtomicStore8",
    "I32AtomicStore16",
    "I64AtomicStore8",
    "I64AtomicStore16",
    "I64AtomicStore32",
    "I32AtomicRmwAdd",
    "I64AtomicRmwAdd",
    "I32AtomicRmw8AddU",
    "I32AtomicRmw16AddU",
    "I64AtomicRmw8AddU",
    "I64AtomicRmw16AddU",
    "I64AtomicRmw32AddU",
    "I32AtomicRmwSub",
    "I64AtomicRmwSub",
    "I32AtomicRmw8SubU",
    "I32AtomicRmw16SubU",
    "I64AtomicRmw8SubU",
    "I64AtomicRmw16SubU",
    "I64AtomicRmw32SubU",
    "I32AtomicRmwAnd",
    "I64AtomicRmwAnd",
    "I32AtomicRmw8AndU",
    "I32AtomicRmw16AndU",
    "I64AtomicRmw8AndU",
    "I64AtomicRmw16AndU",
    "I64AtomicRmw32AndU",
    "I32AtomicRmwOr",
    "I64AtomicRmwOr",
    "I32AtomicRmw8OrU",
    "I32AtomicRmw16OrU",
    "I64AtomicRmw8OrU",
    "I64AtomicRmw16OrU",
    "I64AtomicRmw32OrU",
    "I32AtomicRmwXor",
    "I64AtomicRmwXor",
    "I32AtomicRmw8XorU",
    "I32AtomicRmw16XorU",
    "I64AtomicRmw8XorU",
    "I64AtomicRmw16XorU",
    "I64AtomicRmw32XorU",
    "I32AtomicRmwXchg",
    "I64AtomicRmwXchg",
    "I32AtomicRmw8XchgU",
    "I32AtomicRmw16XchgU",
    "I64AtomicRmw8XchgU",
    "I64AtomicRmw16XchgU",
    "I64AtomicRmw32XchgU",
    "I32AtomicRmwCmpxchg",
    "I64AtomicRmwCmpxchg",
    "I32AtomicRmw8CmpxchgU",
    "I32AtomicRmw16CmpxchgU",
    "I64AtomicRmw8CmpxchgU",
    "I64AtomicRmw16CmpxchgU",
    "I64AtomicRmw32CmpxchgU",
    "V128Load",
    "V128Load8x8S",
    "V128Load8x8U",
    "V128Load16x4S",
    "V128Load16x4U",
    "V128Load32x2S",
    "V128Load32x2U",
    "V128Load8Splat",
    "V128Load16Splat",
    "V128Load32Splat",
    "V128Load64Splat",
    "V128Load32Zero",
    "V128Load64Zero",
    "V128Store",
    "V128Load8Lane",
    "V128Load16Lane",
    "V128Load32Lane",
    "V128Load64Lane",
    "V128Store8Lane",
    "V128Store16Lane",
    "V128Store32Lane",
    "V128Store64Lane",
    "V128Const",
    "I8x16Shuffle",
    "I8x16ExtractLaneS",
    "I8x16ExtractLaneU",
    "I8x16ReplaceLane",
    "I16x8ExtractLaneS",
    "I16x8ExtractLaneU",
    "I16x8ReplaceLane",
    "I32x4ExtractLane",
    "I32x4ReplaceLane",
    "I64x2ExtractLane",
    "I64x2ReplaceLane",
    "F32x4ExtractLane",
    "F32x4ReplaceLane",
    "F64x2ExtractLane",
    "F64x2ReplaceLane",
    "I8x16Swizzle",
    "I8x16Splat",
    "I16x8Splat",
    "I32x4Splat",
    "I64x2Splat",
    "F32x4Splat",
    "F64x2Splat",
    "I8x16Eq",
    "I8x16Ne",
    "I8x16LtS",
    "I8x16LtU",
    "I8x16GtS",
    "I8x16GtU",
    "I8x16LeS",
    "I8x16LeU",
    "I8x16GeS",
    "I8x16GeU",
    "I16x8Eq",
    "I16x8Ne",
    "I16x8LtS",
    "I16x8LtU",
    "I16x8GtS",
    "I16x8GtU",
    "I16x8LeS",
    "I16x8LeU",
    "I16x8GeS",
    "I16x8GeU",
    "I32x4Eq",
    "I32x4Ne",
    "I32x4LtS",
    "I32x4LtU",
    "I32x4GtS",
    "I32x4GtU",
    "I32x4LeS",
    "I32x4LeU",
    "I32x4GeS",
    "I32x4GeU",
    "I64x2Eq",
    "I64x2Ne",
    "I64x2LtS",
    "I64x2GtS",
    "I64x2LeS",
    "I64x2GeS",
    "F32x4Eq",
    "F32x4Ne",
    "F32x4Lt",
    "F32x4Gt",
    "F32x4Le",
    "F32x4Ge",
    "F64x2Eq",
    "F64x2Ne",
    "F64x2Lt",
    "F64x2Gt",
    "F64x2Le",
    "F64x2Ge",
    "V128Not",
    "V128And",
    "V128AndNot",
    "V128Or",
    "V128Xor",
    "V128Bitselect",
    "V128AnyTrue",
    "I8x16Abs",
    "I8x16Neg",
    "I8x16Popcnt",
    "I8x16AllTrue",
    "I8x16Bitmask",
    "I8x16NarrowI16x8S",
    "I8x16NarrowI16x8U",
    "I8x16Shl",
    "I8x16ShrS",
    "I8x16ShrU",
    "I8x16Add",
    "I8x16AddSatS",
    "I8x16AddSatU",
    "I8x16Sub",
    "I8x16SubSatS",
    "I8x16SubSatU",
    "I8x16MinS",
    "I8x16MinU",
    "I8x16MaxS",
    "I8x16MaxU",
    "I8x16RoundingAverageU",
    "I16x8ExtAddPairwiseI8x16S",
    "I16x8ExtAddPairwiseI8x16U",
    "I16x8Abs",
    "I16x8Neg",
    "I16x8Q15MulrSatS",
    "I16x8AllTrue",
    "I16x8Bitmask",
    "I16x8NarrowI32x4S",
    "I16x8NarrowI32x4U",
    "I16x8ExtendLowI8x16S",
    "I16x8ExtendHighI8x16S",
    "I16x8ExtendLowI8x16U",
    "I16x8ExtendHighI8x16U",
    "I16x8Shl",
    "I16x8ShrS",
    "I16x8ShrU",
    "I16x8Add",
    "I16x8AddSatS",
    "I16x8AddSatU",
    "I16x8Sub",
    "I16x8SubSatS",
    "I16x8SubSatU",
    "I16x8Mul",
    "I16x8MinS",
    "I16x8MinU",
    "I16x8MaxS",
    "I16x8MaxU",
    "I16x8RoundingAverageU",
    "I16x8ExtMulLowI8x16S",
    "I16x8ExtMulHighI8x16S",
    "I16x8ExtMulLowI8x16U",
    "I16x8ExtMulHighI8x16U",
    "I32x4ExtAddPairwiseI16x8S",
    "I32x4ExtAddPairwiseI16x8U",
    "I32x4Abs",
    "I32x4Neg",
    "I32x4AllTrue",
    "I32x4Bitmask",
    "I32x4ExtendLowI16x8S",
    "I32x4ExtendHighI16x8S",
    "I32x4ExtendLowI16x8U",
    "I32x4ExtendHighI16x8U",
    "I32x4Shl",
    "I32x4ShrS",
    "I32x4ShrU",
    "I32x4Add",
    "I32x4Sub",
    "I32x4Mul",
    "I32x4MinS",
    "I32x4MinU",
    "I32x4MaxS",
    "I32x4MaxU",
    "I32x4DotI16x8S",
    "I32x4ExtMulLowI16x8S",
    "I32x4ExtMulHighI16x8S",
    "I32x4ExtMulLowI16x8U",
    "I32x4ExtMulHighI16x8U",
    "I64x2Abs",
    "I64x2Neg",
    "I64x2AllTrue",
    "I64x2Bitmask",
    "I64x2ExtendLowI32x4S",
    "I64x2ExtendHighI32x4S",
    "I64x2ExtendLowI32x4U",
    "I64x2ExtendHighI32x4U",
    "I64x2Shl",
    "I64x2ShrS",
    "I64x2ShrU",
    "I64x2Add",
    "I64x2Sub",
    "I64x2Mul",
    "I64x2ExtMulLowI32x4S",
    "I64x2ExtMulHighI32x4S",
    "I64x2ExtMulLowI32x4U",
    "I64x2ExtMulHighI32x4U",
    "F32x4Ceil",
    "F32x4Floor",
    "F32x4Trunc",
    "F32x4Nearest",
    "F32x4Abs",
    "F32x4Neg",
    "F32x4Sqrt",
    "F32x4Add",
    "F32x4Sub",
    "F32x4Mul",
    "F32x4Div",
    "F32x4Min",
    "F32x4Max",
    "F32x4PMin",
    "F32x4PMax",
    "F64x2Ceil",
    "F64x2Floor",
    "F64x2Trunc",
    "F64x2Nearest",
    "F64x2Abs",
    "F64x2Neg",
    "F64x2Sqrt",
    "F64x2Add",
    "F64x2Sub",
    "F64x2Mul",
    "F64x2Div",
    "F64x2Min",
    "F64x2Max",
    "F64x2PMin",
    "F64x2PMax",
    "I32x4TruncSatF32x4S",
    "I32x4TruncSatF32x4U",
    "F32x4ConvertI32x4S",
    "F32x4ConvertI32x4U",
    "I32x4TruncSatF64x2SZero",
    "I32x4TruncSatF64x2UZero",
    "F64x2ConvertLowI32x4S",
    "F64x2ConvertLowI32x4U",
    "F32x4DemoteF64x2Zero",
    "F64x2PromoteLowF32x4",
    "I8x16RelaxedSwizzle",
    "I32x4RelaxedTruncSatF32x4S",
    "I32x4RelaxedTruncSatF32x4U",
    "I32x4RelaxedTruncSatF64x2SZero",
    "I32x4RelaxedTruncSatF64x2UZero",
    "F32x4Fma",
    "F32x4Fms",
    "F64x2Fma",
    "F64x2Fms",
    "I8x16LaneSelect",
    "I16x8LaneSelect",
    "I32x4LaneSelect",
    "I64x2LaneSelect",
    "F32x4RelaxedMin",
    "F32x4RelaxedMax",
    "F64x2RelaxedMin",
    "F64x2RelaxedMax",
];

// From https://github.com/WebAssembly/threads/blob/main/proposals/threads/Overview.md
#[cfg(feature = "gas_calibration")]
pub(crate) const _OPERATOR_THREAD: [&str; 67] = [
    // Load/Store
    "I32AtomicLoad8U",
    "I32AtomicLoad16U",
    "I32AtomicLoad",
    "I64AtomicLoad8U",
    "I64AtomicLoad16U",
    "I64AtomicLoad32U",
    "I64AtomicLoad",
    "I32AtomicStore8",
    "I32AtomicStore16",
    "I32AtomicStore",
    "I64AtomicStore8",
    "I64AtomicStore16",
    "I64AtomicStore32",
    "I64AtomicStore",
    // Read Modify Write
    "I32AtomicRmw8AddU",
    "I32AtomicRmw16AddU",
    "I32AtomicRmwAdd",
    "I64AtomicRmw8AddU",
    "I64AtomicRmw16AddU",
    "I64AtomicRmw32AddU",
    "I64AtomicRmwAdd",
    "I32AtomicRmw8SubU",
    "I32AtomicRmw16SubU",
    "I32AtomicRmwSub",
    "I64AtomicRmw8SubU",
    "I64AtomicRmw16SubU",
    "I64AtomicRmw32SubU",
    "I64AtomicRmwSub",
    "I32AtomicRmw8AndU",
    "I32AtomicRmw16AndU",
    "I32AtomicRmwAnd",
    "I64AtomicRmw8AndU",
    "I64AtomicRmw16AndU",
    "I64AtomicRmw32AndU",
    "I64AtomicRmwAnd",
    "I32AtomicRmw8OrU",
    "I32AtomicRmw16OrU",
    "I32AtomicRmwOr",
    "I64AtomicRmw8OrU",
    "I64AtomicRmw16OrU",
    "I64AtomicRmw32OrU",
    "I64AtomicRmwOr",
    "I32AtomicRmw8XorU",
    "I32AtomicRmw16XorU",
    "I32AtomicRmwXor",
    "I64AtomicRmw8XorU",
    "I64AtomicRmw16XorU",
    "I64AtomicRmw32XorU",
    "I64AtomicRmwXor",
    "I32AtomicRmw8XchgU",
    "I32AtomicRmw16XchgU",
    "I32AtomicRmwXchg",
    "I64AtomicRmw8XchgU",
    "I64AtomicRmw16XchgU",
    "I64AtomicRmw32XchgU",
    "I64AtomicRmwXchg",
    // Compare Exchange
    "I32AtomicRmw8CmpxchgU",
    "I32AtomicRmw16CmpxchgU",
    "I32AtomicRmwCmpxchg",
    "I64AtomicRmw8CmpxchgU",
    "I64AtomicRmw16CmpxchgU",
    "I64AtomicRmw32CmpxchgU",
    "I64AtomicRmwCmpxchg",
    // Wait / Notify / Fence
    "MemoryAtomicNotify",
    "MemoryAtomicWait32",
    "MemoryAtomicWait64",
    "AtomicFence",
];

// From https://webassembly.github.io/spec/core/_download/WebAssembly.pdf
// Section 7.6 Change History -> 7.6.1 Release 2.0 -> Non-trapping float-to-int
// conversions https://github.com/WebAssembly/spec/blob/main/proposals/nontrapping-float-to-int-conversion/Overview.md
#[cfg(feature = "gas_calibration")]
pub(crate) const _OPERATOR_NON_TRAPPING_FLOAT_TO_INT: [&str; 8] = [
    "I32TruncSatF32S",
    "I32TruncSatF32U",
    "I32TruncSatF64S",
    "I32TruncSatF64U",
    "I64TruncSatF32S",
    "I64TruncSatF32U",
    "I64TruncSatF64S",
    "I64TruncSatF64U",
];

// From https://webassembly.github.io/spec/core/_download/WebAssembly.pdf
// Section 7.6 Change History -> 7.6.1 Release 2.0 -> Bulk memory
#[cfg(feature = "gas_calibration")]
pub(crate) const _OPERATOR_BULK_MEMORY: [&str; 8] = [
    "MemoryFill",
    "MemoryInit",
    "MemoryCopy",
    "DataDrop",
    "TableFill",
    "TableInit",
    "TableCopy",
    "ElemDrop",
];

#[cfg(feature = "gas_calibration")]
pub(crate) const _OPERATOR_VECTOR: [&str; 236] = [
    "V128Load",
    "V128Load8x8S",
    "V128Load8x8U",
    "V128Load16x4S",
    "V128Load16x4U",
    "V128Load32x2S",
    "V128Load32x2U",
    "V128Load8Splat",
    "V128Load16Splat",
    "V128Load32Splat",
    "V128Load64Splat",
    "V128Store",
    "V128Const",
    "I8x16Shuffle",
    "I8x16Swizzle",
    "I8x16Splat",
    "I16x8Splat",
    "I32x4Splat",
    "I64x2Splat",
    "F32x4Splat",
    "F64x2Splat",
    "I8x16ExtractLaneS",
    "I8x16ExtractLaneU",
    "I8x16ReplaceLane",
    "I16x8ExtractLaneS",
    "I16x8ExtractLaneU",
    "I16x8ReplaceLane",
    "I32x4ExtractLane",
    "I32x4ReplaceLane",
    "I64x2ExtractLane",
    "I64x2ReplaceLane",
    "F32x4ExtractLane",
    "F32x4ReplaceLane",
    "F64x2ExtractLane",
    "F64x2ReplaceLane",
    "I8x16Eq",
    "I8x16Ne",
    "I8x16LtS",
    "I8x16LtU",
    "I8x16GtS",
    "I8x16GtU",
    "I8x16LeS",
    "I8x16LeU",
    "I8x16GeS",
    "I8x16GeU",
    "I16x8Eq",
    "I16x8Ne",
    "I16x8LtS",
    "I16x8LtU",
    "I16x8GtS",
    "I16x8GtU",
    "I16x8LeS",
    "I16x8LeU",
    "I16x8GeS",
    "I16x8GeU",
    "I32x4Eq",
    "I32x4Ne",
    "I32x4LtS",
    "I32x4LtU",
    "I32x4GtS",
    "I32x4GtU",
    "I32x4LeS",
    "I32x4LeU",
    "I32x4GeS",
    "I32x4GeU",
    "F32x4Eq",
    "F32x4Ne",
    "F32x4Lt",
    "F32x4Gt",
    "F32x4Le",
    "F32x4Ge",
    "F64x2Eq",
    "F64x2Ne",
    "F64x2Lt",
    "F64x2Gt",
    "F64x2Le",
    "F64x2Ge",
    "V128Not",
    "V128And",
    "V128AndNot",
    "V128Or",
    "V128Xor",
    "V128Bitselect",
    "I8x16Abs",
    "I8x16Neg",
    "I8x16AllTrue",
    "I8x16Bitmask",
    "I8x16NarrowI16x8S",
    "I8x16NarrowI16x8U",
    "I8x16Shl",
    "I8x16ShrS",
    "I8x16ShrU",
    "I8x16Add",
    "I8x16AddSatS",
    "I8x16AddSatU",
    "I8x16Sub",
    "I8x16SubSatS",
    "I8x16SubSatU",
    "I8x16MinS",
    "I8x16MinU",
    "I8x16MaxS",
    "I8x16MaxU",
    "I8x16RoundingAverageU", // "I8x16AvgrU",
    "I16x8Abs",
    "I16x8Neg",
    "I16x8AllTrue",
    "I16x8Bitmask",
    "I16x8NarrowI32x4S",
    "I16x8NarrowI32x4U",
    "I16x8ExtendLowI8x16S",
    "I16x8ExtendHighI8x16S",
    "I16x8ExtendLowI8x16U",
    "I16x8ExtendHighI8x16U",
    "I16x8Shl",
    "I16x8ShrS",
    "I16x8ShrU",
    "I16x8Add",
    "I16x8AddSatS",
    "I16x8AddSatU",
    "I16x8Sub",
    "I16x8SubSatS",
    "I16x8SubSatU",
    "I16x8Mul",
    "I16x8MinS",
    "I16x8MinU",
    "I16x8MaxS",
    "I16x8MaxU",
    "I16x8RoundingAverageU", // "I16x8AvgrU",
    "I16x8Q15MulrSatS",      // "I16x8Q15mulrSatS",
    "I32x4Abs",
    "I32x4Neg",
    "I32x4AllTrue",
    "I32x4Bitmask",
    "I32x4ExtendLowI16x8S",
    "I32x4ExtendHighI16x8S",
    "I32x4ExtendLowI16x8U",
    "I32x4ExtendHighI16x8U",
    "I32x4Shl",
    "I32x4ShrS",
    "I32x4ShrU",
    "I32x4Add",
    "I32x4Sub",
    "I32x4Mul",
    "I32x4MinS",
    "I32x4MinU",
    "I32x4MaxS",
    "I32x4MaxU",
    "I32x4DotI16x8S",
    "I64x2Eq",
    "I64x2Abs",
    "I64x2Neg",
    "I64x2AllTrue",
    "I64x2Bitmask",
    "I64x2Shl",
    "I64x2ShrS",
    "I64x2ShrU",
    "I64x2Add",
    "I64x2Sub",
    "I64x2Mul",
    "I64x2ExtendLowI32x4S",
    "I64x2ExtendHighI32x4S",
    "I64x2ExtendLowI32x4U",
    "I64x2ExtendHighI32x4U",
    "F32x4Abs",
    "F32x4Neg",
    "F32x4Sqrt",
    "F32x4Add",
    "F32x4Sub",
    "F32x4Mul",
    "F32x4Div",
    "F32x4Min",
    "F32x4Max",
    "F32x4PMin", // "F32x4Pmin",
    "F32x4PMax", // "F32x4Pmax",
    "F32x4Ceil",
    "F32x4Floor",
    "F32x4Trunc",
    "F32x4Nearest",
    "F64x2Abs",
    "F64x2Neg",
    "F64x2Sqrt",
    "F64x2Add",
    "F64x2Sub",
    "F64x2Mul",
    "F64x2Div",
    "F64x2Min",
    "F64x2Max",
    "F64x2PMin", // "F64x2Pmin",
    "F64x2PMax", // "F64x2Pmax",
    "F64x2Ceil",
    "F64x2Floor",
    "F64x2Trunc",
    "F64x2Nearest",
    "I32x4TruncSatF32x4S",
    "I32x4TruncSatF32x4U",
    "F32x4ConvertI32x4S",
    "F32x4ConvertI32x4U",
    "V128Load32Zero",
    "V128Load64Zero",
    "I16x8ExtMulLowI8x16S", //
    "I16x8ExtMulHighI8x16S",
    "I16x8ExtMulLowI8x16U",
    "I16x8ExtMulHighI8x16U",
    "I32x4ExtMulLowI16x8S",
    "I32x4ExtMulHighI16x8S",
    "I32x4ExtMulLowI16x8U",
    "I32x4ExtMulHighI16x8U",
    "I64x2ExtMulLowI32x4S",
    "I64x2ExtMulHighI32x4S",
    "I64x2ExtMulLowI32x4U",
    "I64x2ExtMulHighI32x4U", //
    "V128AnyTrue",
    "V128Load8Lane",
    "V128Load16Lane",
    "V128Load32Lane",
    "V128Load64Lane",
    "V128Store8Lane",
    "V128Store16Lane",
    "V128Store32Lane",
    "V128Store64Lane",
    "I64x2Ne",
    "F64x2ConvertLowI32x4S",
    "F64x2ConvertLowI32x4U",
    "I32x4TruncSatF64x2SZero",
    "I32x4TruncSatF64x2UZero",
    "F32x4DemoteF64x2Zero",
    "F64x2PromoteLowF32x4",
    "I8x16Popcnt",
    "I16x8ExtAddPairwiseI8x16S",
    "I16x8ExtAddPairwiseI8x16U",
    "I32x4ExtAddPairwiseI16x8S",
    "I32x4ExtAddPairwiseI16x8U",
    "I64x2LtS",
    "I64x2GtS",
    "I64x2LeS",
    "I64x2GeS",
];

#[cfg(all(feature = "gas_calibration", test))]
pub const OPERATOR_CARDINALITY: usize = OPERATOR_VARIANTS.len();
