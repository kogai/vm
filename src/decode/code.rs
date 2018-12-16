use std::convert::From;

#[derive(Debug, PartialEq, Clone)]
pub enum Code {
  Reserved, // Reserved code.
  Unreachable,
  Nop,
  Block,
  Loop,
  If,
  Else,
  End,
  Br,
  BrIf,
  BrTable,
  Return,
  Call,
  CallIndirect,

  Select,
  DropInst,

  ConstI32,
  ConstI64,
  F32Const,
  F64Const,

  GetLocal,
  TeeLocal,
  SetLocal,
  GetGlobal,
  SetGlobal,

  I32Load,
  I64Load,
  F32Load,
  F64Load,
  I32Load8Sign,
  I32Load8Unsign,
  I32Load16Sign,
  I32Load16Unsign,
  I64Load8Sign,
  I64Load8Unsign,
  I64Load16Sign,
  I64Load16Unsign,
  I64Load32Sign,
  I64Load32Unsign,
  I32Store,
  I64Store,
  F32Store,
  F64Store,
  I32Store8,
  I32Store16,
  I64Store8,
  I64Store16,
  I64Store32,
  MemorySize,
  MemoryGrow,

  I32CountLeadingZero,
  I32CountTrailingZero,
  I32CountNonZero,
  I32Add,
  I32Sub,
  I32Mul,
  I32WrapI64,
  I32DivSign,
  I32DivUnsign,
  I32RemSign,
  I32RemUnsign,
  I32And,
  I32Or,
  I32Xor,
  I32ShiftLeft,
  I32ShiftRIghtSign,
  I32ShiftRightUnsign,
  I32RotateLeft,
  I32RotateRight,

  I64CountLeadingZero,
  I64CountTrailingZero,
  I64CountNonZero,
  I64Add,
  I64Sub,
  I64Mul,
  I64DivSign,
  I64DivUnsign,
  I64RemSign,
  I64RemUnsign,
  I64And,
  I64Or,
  I64Xor,
  I64ShiftLeft,
  I64ShiftRightSign,
  I64ShiftRightUnsign,
  I64RotateLeft,
  I64RotateRight,

  I32EqualZero,
  // TODO: Add prefix to indicate data-type like I32
  Equal,
  NotEqual,
  LessThanSign,
  LessThanUnsign,
  GreaterThanSign,
  I32GreaterThanUnsign,
  I32LessEqualSign,
  I32LessEqualUnsign,
  I32GreaterEqualSign,
  I32GreaterEqualUnsign,

  I64EqualZero,
  I64Equal,
  I64NotEqual,
  I64LessThanSign,
  I64LessThanUnSign,
  I64GreaterThanSign,
  I64GreaterThanUnSign,
  I64LessEqualSign,
  I64LessEqualUnSign,
  I64GreaterEqualSign,
  I64GreaterEqualUnSign,

  F32Equal,
  F32NotEqual,
  F32LessThan,
  F32GreaterThan,
  F32LessEqual,
  F32GreaterEqual,
  F64Equal,
  F64NotEqual,
  F64LessThan,
  F64GreaterThan,
  F64LessEqual,
  F64GreaterEqual,

  F32Abs,
  F32Neg,
  F32Ceil,
  F32Floor,
  F32Trunc,
  F32Nearest,
  F32Sqrt,
  F32Add,
  F32Sub,
  F32Mul,
  F32Div,
  F32Min,
  F32Max,
  F32Copysign,

  F64Abs,
  F64Neg,
  F64Ceil,
  F64Floor,
  F64Trunc,
  F64Nearest,
  F64Sqrt,
  F64Add,
  F64Sub,
  F64Mul,
  F64Div,
  F64Min,
  F64Max,
  F64Copysign,

  I32TruncSignF32,
  I32TruncUnsignF32,
  I32TruncSignF64,
  I32TruncUnsignF64,
  I64ExtendSignI32,
  I64ExtendUnsignI32,
  I64TruncSignF32,
  I64TruncUnsignF32,
  I64TruncSignF64,
  I64TruncUnsignF64,
  F32ConvertSignI32,
  F32ConvertUnsignI32,
  F32ConvertSignI64,
  F32ConvertUnsignI64,
  F32DemoteF64,
  F64ConvertSignI32,
  F64ConvertUnsignI32,
  F64ConvertSignI64,
  F64ConvertUnsignI64,
  F64PromoteF32,
  I32ReinterpretF32,
  I64ReinterpretF64,
  F32ReinterpretI32,
  F64ReinterpretI64,
}

impl From<Option<u8>> for Code {
  fn from(code: Option<u8>) -> Self {
    use self::Code::*;
    match code {
      Some(0x0) => Unreachable,
      Some(0x1) => Nop,
      Some(0x2) => Block,
      Some(0x3) => Loop,
      Some(0x4) => If,
      Some(0x5) => Else,
      Some(0x06) | Some(0x07) | Some(0x08) | Some(0x09) | Some(0x0A) => Reserved,
      Some(0x0b) => End,
      Some(0x0C) => Br,
      Some(0x0D) => BrIf,
      Some(0x0e) => BrTable,
      Some(0x0f) => Return,
      Some(0x10) => Call,
      Some(0x11) => CallIndirect,
      Some(0x12) | Some(0x13) | Some(0x14) | Some(0x15) | Some(0x16) | Some(0x17) | Some(0x18)
      | Some(0x19) => Reserved,
      Some(0x1a) => DropInst,
      Some(0x1b) => Select,
      Some(0x20) => GetLocal,
      Some(0x21) => SetLocal,
      Some(0x22) => TeeLocal,
      Some(0x23) => GetGlobal,
      Some(0x24) => SetGlobal,
      Some(0x25) | Some(0x26) | Some(0x27) => Reserved,

      Some(0x28) => I32Load,
      Some(0x29) => I64Load,
      Some(0x2a) => F32Load,
      Some(0x2b) => F64Load,
      Some(0x2c) => I32Load8Sign,
      Some(0x2d) => I32Load8Unsign,
      Some(0x2e) => I32Load16Sign,
      Some(0x2f) => I32Load16Unsign,
      Some(0x30) => I64Load8Sign,
      Some(0x31) => I64Load8Unsign,
      Some(0x32) => I64Load16Sign,
      Some(0x33) => I64Load16Unsign,
      Some(0x34) => I64Load32Sign,
      Some(0x35) => I64Load32Unsign,
      Some(0x36) => I32Store,
      Some(0x37) => I64Store,
      Some(0x38) => F32Store,
      Some(0x39) => F64Store,
      Some(0x3a) => I32Store8,
      Some(0x3b) => I32Store16,
      Some(0x3c) => I64Store8,
      Some(0x3d) => I64Store16,
      Some(0x3e) => I64Store32,
      Some(0x3f) => MemorySize,
      Some(0x40) => MemoryGrow,

      Some(0x41) => ConstI32,
      Some(0x42) => ConstI64,
      Some(0x43) => F32Const,
      Some(0x44) => F64Const,
      Some(0x45) => I32EqualZero,
      Some(0x46) => Equal,
      Some(0x47) => NotEqual,
      Some(0x48) => LessThanSign,
      Some(0x49) => LessThanUnsign,
      Some(0x4a) => GreaterThanSign,
      Some(0x4b) => I32GreaterThanUnsign,
      Some(0x4c) => I32LessEqualSign,
      Some(0x4d) => I32LessEqualUnsign,
      Some(0x4e) => I32GreaterEqualSign,
      Some(0x4f) => I32GreaterEqualUnsign,
      Some(0x50) => I64EqualZero,
      Some(0x51) => I64Equal,
      Some(0x52) => I64NotEqual,
      Some(0x53) => I64LessThanSign,
      Some(0x54) => I64LessThanUnSign,
      Some(0x55) => I64GreaterThanSign,
      Some(0x56) => I64GreaterThanUnSign,
      Some(0x57) => I64LessEqualSign,
      Some(0x58) => I64LessEqualUnSign,
      Some(0x59) => I64GreaterEqualSign,
      Some(0x5a) => I64GreaterEqualUnSign,

      Some(0x5B) => F32Equal,
      Some(0x5C) => F32NotEqual,
      Some(0x5D) => F32LessThan,
      Some(0x5E) => F32GreaterThan,
      Some(0x5F) => F32LessEqual,
      Some(0x60) => F32GreaterEqual,
      Some(0x61) => F64Equal,
      Some(0x62) => F64NotEqual,
      Some(0x63) => F64LessThan,
      Some(0x64) => F64GreaterThan,
      Some(0x65) => F64LessEqual,
      Some(0x66) => F64GreaterEqual,

      Some(0x67) => I32CountLeadingZero,
      Some(0x68) => I32CountTrailingZero,
      Some(0x69) => I32CountNonZero,
      Some(0x6a) => I32Add,
      Some(0x6b) => I32Sub,
      Some(0x6c) => I32Mul,
      Some(0x6d) => I32DivSign,
      Some(0x6e) => I32DivUnsign,
      Some(0x6f) => I32RemSign,
      Some(0x70) => I32RemUnsign,
      Some(0x71) => I32And,
      Some(0x72) => I32Or,
      Some(0x73) => I32Xor,
      Some(0x74) => I32ShiftLeft,
      Some(0x75) => I32ShiftRIghtSign,
      Some(0x76) => I32ShiftRightUnsign,
      Some(0x77) => I32RotateLeft,
      Some(0x78) => I32RotateRight,
      Some(0x79) => I64CountLeadingZero,
      Some(0x7a) => I64CountTrailingZero,
      Some(0x7b) => I64CountNonZero,
      Some(0x7c) => I64Add,
      Some(0x7d) => I64Sub,
      Some(0x7e) => I64Mul,
      Some(0x7f) => I64DivSign,
      Some(0x80) => I64DivUnsign,
      Some(0x81) => I64RemSign,
      Some(0x82) => I64RemUnsign,
      Some(0x83) => I64And,
      Some(0x84) => I64Or,
      Some(0x85) => I64Xor,
      Some(0x86) => I64ShiftLeft,
      Some(0x87) => I64ShiftRightSign,
      Some(0x88) => I64ShiftRightUnsign,
      Some(0x89) => I64RotateLeft,
      Some(0x8a) => I64RotateRight,

      Some(0x8b) => F32Abs,
      Some(0x8c) => F32Neg,
      Some(0x8d) => F32Ceil,
      Some(0x8e) => F32Floor,
      Some(0x8f) => F32Trunc,
      Some(0x90) => F32Nearest,
      Some(0x91) => F32Sqrt,
      Some(0x92) => F32Add,
      Some(0x93) => F32Sub,
      Some(0x94) => F32Mul,
      Some(0x95) => F32Div,
      Some(0x96) => F32Min,
      Some(0x97) => F32Max,
      Some(0x98) => F32Copysign,

      Some(0x99) => F64Abs,
      Some(0x9a) => F64Neg,
      Some(0x9b) => F64Ceil,
      Some(0x9c) => F64Floor,
      Some(0x9d) => F64Trunc,
      Some(0x9e) => F64Nearest,
      Some(0x9f) => F64Sqrt,
      Some(0xa0) => F64Add,
      Some(0xa1) => F64Sub,
      Some(0xa2) => F64Mul,
      Some(0xa3) => F64Div,
      Some(0xa4) => F64Min,
      Some(0xa5) => F64Max,
      Some(0xa6) => F64Copysign,
      Some(0xa7) => I32WrapI64,
      Some(0xa8) => I32TruncSignF32,
      Some(0xa9) => I32TruncUnsignF32,
      Some(0xaa) => I32TruncSignF64,
      Some(0xab) => I32TruncUnsignF64,
      Some(0xac) => I64ExtendSignI32,
      Some(0xad) => I64ExtendUnsignI32,
      Some(0xae) => I64TruncSignF32,
      Some(0xaf) => I64TruncUnsignF32,
      Some(0xb0) => I64TruncSignF64,
      Some(0xb1) => I64TruncUnsignF64,
      Some(0xb2) => F32ConvertSignI32,
      Some(0xb3) => F32ConvertUnsignI32,
      Some(0xb4) => F32ConvertSignI64,
      Some(0xb5) => F32ConvertUnsignI64,
      Some(0xb6) => F32DemoteF64,
      Some(0xb7) => F64ConvertSignI32,
      Some(0xb8) => F64ConvertUnsignI32,
      Some(0xb9) => F64ConvertSignI64,
      Some(0xba) => F64ConvertUnsignI64,
      Some(0xbb) => F64PromoteF32,
      Some(0xbc) => I32ReinterpretF32,
      Some(0xbd) => I64ReinterpretF64,
      Some(0xbe) => F32ReinterpretI32,
      Some(0xbf) => F64ReinterpretI64,
      x => unreachable!("Code {:x?} does not supported yet.", x),
    }
  }
}

impl Code {
  pub fn is_else_or_end(code: Option<u8>) -> bool {
    match code {
      Some(0x5) | Some(0x0b) => true,
      _ => false,
    }
  }
}

#[derive(Debug)]
pub enum ExportDescriptionCode {
  ExportDescFunctionIdx,
  ExportDescTableIdx,
  ExportDescMemIdx,
  ExportDescGlobalIdx,
}

impl From<Option<u8>> for ExportDescriptionCode {
  fn from(code: Option<u8>) -> Self {
    use self::ExportDescriptionCode::*;
    match code {
      Some(0x00) => ExportDescFunctionIdx,
      Some(0x01) => ExportDescTableIdx,
      Some(0x02) => ExportDescMemIdx,
      Some(0x03) => ExportDescGlobalIdx,
      x => unreachable!("Export description code {:x?} does not supported yet.", x),
    }
  }
}