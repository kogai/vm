use super::error::{Result, TypeError};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use decode::{Element, Section, TableType};
use function::FunctionType;
use global::GlobalType;
use inst::{Indice, Inst};
use memory::Limit;
// use module::{FUNCTION_DESCRIPTOR, GLOBAL_DESCRIPTOR, MEMORY_DESCRIPTOR, TABLE_DESCRIPTOR};
// use trap::Trap;
// use value::Values;
use value_type::ValueTypes;

type ResultType = [ValueTypes; 1];

#[derive(Debug, Clone)]
enum Entry {
  Type(ValueTypes),
  Label,
}

#[derive(Debug)]
struct TypeStack(RefCell<Vec<Entry>>);

impl TypeStack {
  fn new() -> Self {
    TypeStack(RefCell::new(Vec::new()))
  }

  fn push(&self, ty: ValueTypes) {
    self.0.borrow_mut().push(Entry::Type(ty));
  }

  fn push_label(&self) {
    self.0.borrow_mut().push(Entry::Label);
  }

  fn pop(&self) -> Option<Entry> {
    self.0.borrow_mut().pop()
  }

  fn pop_type(&self) -> Result<ValueTypes> {
    match self.pop() {
      Some(Entry::Type(ty)) => Ok(ty),
      _ => Err(TypeError::TypeMismatch),
    }
  }

  fn pop_until_label(&self) -> Result<Vec<Entry>> {
    let mut buf = Vec::new();
    while let Some(Entry::Type(ty)) = self.pop() {
      buf.push(Entry::Type(ty));
    }
    Ok(buf)
  }

  fn pop_i32(&self) -> Result<ValueTypes> {
    match self.0.borrow_mut().pop() {
      Some(Entry::Type(ValueTypes::I32)) => Ok(ValueTypes::I32),
      _ => Err(TypeError::TypeMismatch),
    }
  }
}

#[derive(Debug)]
struct Function<'a> {
  function_type: &'a FunctionType,
  locals: &'a [ValueTypes],
  body: &'a [Inst],
  body_ptr: Cell<usize>,
  type_stack: TypeStack,
}

impl<'a> Function<'a> {
  fn new(
    function_type: &'a FunctionType,
    locals: &'a [ValueTypes],
    body: &'a [Inst],
  ) -> Function<'a> {
    Function {
      function_type,
      locals,
      body,
      body_ptr: Cell::new(0),
      type_stack: TypeStack::new(),
    }
  }

  fn pop(&self) -> Option<&Inst> {
    let ptr = self.body_ptr.get();
    self.body_ptr.set(ptr + 1);
    self.body.get(ptr)
  }

  fn pop_value_type(&self) -> Option<ValueTypes> {
    match self.pop() {
      Some(Inst::RuntimeValue(ty)) => Some(ty.to_owned()),
      _ => None,
    }
  }
}

pub struct Context<'a> {
  function_types: &'a Vec<FunctionType>,
  functions: Vec<Function<'a>>,
  //  exports: ExternalInterfaces,
  //  codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  //  datas: Vec<Data>,
  limits: &'a Vec<Limit>,
  tables: &'a Vec<TableType>,
  globals: &'a Vec<(GlobalType, Vec<Inst>)>,
  elements: &'a Vec<Element>,
  //  customs: Vec<(String, Vec<u8>)>,
  //  imports: ExternalInterfaces,
  //  start: Option<u32>,
  locals: RefCell<Vec<ValueTypes>>,
  labels: RefCell<VecDeque<ResultType>>,
  return_type: RefCell<ResultType>,
}

macro_rules! bin_op {
  ($stack: ident) => {{
    let l = $stack.pop_type()?;
    let r = $stack.pop_type()?;
    if l != r {
      return Err(TypeError::TypeMismatch);
    }
    $stack.push(l);
  }};
}

macro_rules! test_op {
  ($stack: ident) => {{
    $stack.pop_type()?;
    $stack.push(ValueTypes::I32);
  }};
}

macro_rules! rel_op {
  ($stack: ident) => {{
    let l = $stack.pop_type()?;
    let r = $stack.pop_type()?;
    if l != r {
      return Err(TypeError::TypeMismatch);
    }
    $stack.push(ValueTypes::I32);
  }};
}

impl<'a> Context<'a> {
  pub fn new(module: &'a Section) -> Result<Self> {
    Ok(Context {
      function_types: &module.function_types,
      functions: module
        .codes
        .iter()
        .enumerate()
        .map(|(idx, code)| {
          let idx = module.functions.get(idx).map(|n| Indice::from(*n))?;
          let function_type = module.function_types.get(idx.to_usize())?;
          let (body, locals) = match code {
            Ok((body, locals)) => Ok((body, locals)),
            Err(ref err) => Err(TypeError::Trap(err.to_owned())),
          }?;
          Ok(Function::new(function_type, locals, body))
        })
        .collect::<Result<Vec<_>>>()?,
      globals: &module.globals,
      tables: &module.tables,
      elements: &module.elements,
      limits: &module.limits,

      locals: RefCell::new(Vec::new()),
      labels: RefCell::new(VecDeque::new()),
      return_type: RefCell::new([ValueTypes::Empty; 1]),
    })
  }

  fn validate_constant(&self, expr: &[Inst]) -> Result<ValueTypes> {
    let type_stack = TypeStack::new();
    match expr.get(0) {
      Some(Inst::I32Const(_)) => type_stack.push(ValueTypes::I32),
      Some(Inst::I64Const(_)) => type_stack.push(ValueTypes::I64),
      Some(Inst::F32Const(_)) => type_stack.push(ValueTypes::F32),
      Some(Inst::F64Const(_)) => type_stack.push(ValueTypes::F64),
      Some(Inst::GetGlobal(idx)) => match self.globals.get(*idx as usize) {
        Some((GlobalType::Const(ty), _)) | Some((GlobalType::Var(ty), _)) => {
          type_stack.push(ty.clone())
        }
        _ => return Err(TypeError::TypeMismatch),
      },
      _ => return Err(TypeError::ConstantExpressionRequired),
    };
    match expr.get(1) {
      Some(Inst::End) => {}
      _ => return Err(TypeError::ConstantExpressionRequired),
    };
    type_stack.pop_type()
  }

  fn validate_elements(&self) -> Result<()> {
    for Element {
      table_idx,
      offset,
      init,
    } in self.elements.iter()
    {
      if self.tables.get(*table_idx as usize).is_none() {
        return Err(TypeError::UnknownTable(*table_idx));
      }
      if ValueTypes::I32 != self.validate_constant(offset)? {
        return Err(TypeError::TypeMismatch);
      }
      for i in init.iter() {
        self
          .functions
          .get(*i as usize)
          .ok_or(TypeError::TypeMismatch)?;
      }
    }
    Ok(())
  }

  fn validate_function_types(&self) -> Result<()> {
    for fy in self.function_types.iter() {
      if fy.returns().len() > 1 {
        return Err(TypeError::InvalidResultArity);
      }
    }
    Ok(())
  }

  fn validate_functions(&self) -> Result<()> {
    for f in self.functions.iter() {
      self.validate_function(f)?;
    }
    Ok(())
  }

  fn validate_load(
    &self,
    cxt: &TypeStack,
    align: u32,
    bit_width: u32,
    ty: ValueTypes,
  ) -> Result<()> {
    self.limits.first().ok_or(TypeError::TypeMismatch)?;
    if 2u32.pow(align) > bit_width / 8 {
      return Err(TypeError::InvalidAlignment);
    };
    cxt.pop_i32()?;
    cxt.push(ty);
    Ok(())
  }

  fn validate_store(&self, cxt: &TypeStack, align: u32, bit_width: u32) -> Result<()> {
    self.limits.first().ok_or(TypeError::TypeMismatch)?;
    if 2u32.pow(align) > bit_width / 8 {
      return Err(TypeError::InvalidAlignment);
    };
    cxt.pop_i32()?;
    Ok(())
  }

  fn validate_unary(&self, cxt: &TypeStack) -> Result<()> {
    let t = cxt.pop_type()?;
    cxt.push(t);
    Ok(())
  }

  fn validate_convert(&self, cxt: &TypeStack, from: ValueTypes, to: ValueTypes) -> Result<()> {
    let from_ty = cxt.pop_type()?;
    if from_ty != from {
      return Err(TypeError::TypeMismatch);
    }
    cxt.push(to);
    Ok(())
  }

  fn validate_function(&self, function: &Function) -> Result<()> {
    use self::Inst::*;
    let cxt = &function.type_stack;
    let labels = &mut self.labels.borrow_mut();
    let locals = &mut self.locals.borrow_mut();
    let return_type = &mut self.return_type.borrow_mut();

    labels.push_front(
      [match function.function_type.returns().first() {
        Some(ty) => ty.clone(),
        None => ValueTypes::Empty,
      }; 1],
    );

    while let Some(inst) = function.pop() {
      match inst {
        Unreachable => {}
        Nop => {}
        Block(_) => {
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        Loop(_) => {
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        If(_, _) => {
          let _ = cxt.pop_i32()?;
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        Else => {
          let expect = labels.pop_front().ok_or(TypeError::TypeMismatch)?[0].clone();
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
          cxt.pop_until_label()?;
          labels.push_front([expect; 1]);
        }
        End => {
          let expect = labels.pop_front().ok_or(TypeError::TypeMismatch)?[0].clone();
          match cxt.pop() {
            Some(Entry::Type(actual)) => {
              if expect != actual {
                return Err(TypeError::TypeMismatch);
              };
              cxt.pop_until_label()?;
            }
            _ => {
              if expect != ValueTypes::Empty {
                return Err(TypeError::TypeMismatch);
              }
            }
          };
        }

        Br(idx) => {
          let expect = labels.get(*idx as usize).ok_or(TypeError::UnknownLabel)?[0].clone();
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        BrIf(idx) => {
          let expect = labels.get(*idx as usize).ok_or(TypeError::UnknownLabel)?[0].clone();
          let actual = cxt.pop_type()?;
          cxt.pop_i32()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        BrTable(indices, idx) => {
          let expect = labels.get(*idx as usize).ok_or(TypeError::UnknownLabel)?[0].clone();
          for i in indices.iter() {
            let actual = labels.get(*i as usize).ok_or(TypeError::UnknownLabel)?[0].clone();
            if expect != actual {
              return Err(TypeError::TypeMismatch);
            }
          }
          let actual = cxt.pop_type()?;
          cxt.pop_i32()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        Return => {
          let expect = return_type[0].clone();
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        Call(_) => unimplemented!(),
        CallIndirect(_) => unimplemented!(),

        I32Const(_) => cxt.push(ValueTypes::I32),
        I64Const(_) => cxt.push(ValueTypes::I64),
        F32Const(_) => cxt.push(ValueTypes::F32),
        F64Const(_) => cxt.push(ValueTypes::F64),

        GetLocal(idx) => {
          let actual = locals.get(*idx as usize).ok_or(TypeError::TypeMismatch)?;
          cxt.push(actual.clone());
        }
        SetLocal(idx) => {
          let expect = cxt.pop_type()?;
          let actual = locals.get(*idx as usize).ok_or(TypeError::TypeMismatch)?;
          if &expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        TeeLocal(_) => unimplemented!(),
        GetGlobal(_) => unimplemented!(),
        SetGlobal(_) => unimplemented!(),

        I32Load(align, _) => self.validate_load(cxt, *align, 32, ValueTypes::I32)?,
        I64Load(align, _) => self.validate_load(cxt, *align, 64, ValueTypes::I64)?,
        F32Load(align, _) => self.validate_load(cxt, *align, 32, ValueTypes::F32)?,
        F64Load(align, _) => self.validate_load(cxt, *align, 64, ValueTypes::F64)?,
        I32Load8Sign(align, _) => self.validate_load(cxt, *align, 8, ValueTypes::I32)?,
        I32Load8Unsign(align, _) => self.validate_load(cxt, *align, 8, ValueTypes::I32)?,
        I32Load16Sign(align, _) => self.validate_load(cxt, *align, 16, ValueTypes::I32)?,
        I32Load16Unsign(align, _) => self.validate_load(cxt, *align, 16, ValueTypes::I32)?,
        I64Load8Sign(align, _) => self.validate_load(cxt, *align, 8, ValueTypes::I64)?,
        I64Load8Unsign(align, _) => self.validate_load(cxt, *align, 8, ValueTypes::I64)?,
        I64Load16Sign(align, _) => self.validate_load(cxt, *align, 16, ValueTypes::I64)?,
        I64Load16Unsign(align, _) => self.validate_load(cxt, *align, 16, ValueTypes::I64)?,
        I64Load32Sign(align, _) => self.validate_load(cxt, *align, 32, ValueTypes::I64)?,
        I64Load32Unsign(align, _) => self.validate_load(cxt, *align, 32, ValueTypes::I64)?,

        I32Store(align, _) => self.validate_store(cxt, *align, 32)?,
        I64Store(align, _) => self.validate_store(cxt, *align, 64)?,
        F32Store(align, _) => self.validate_store(cxt, *align, 32)?,
        F64Store(align, _) => self.validate_store(cxt, *align, 64)?,
        I32Store8(align, _) => self.validate_store(cxt, *align, 8)?,
        I32Store16(align, _) => self.validate_store(cxt, *align, 16)?,
        I64Store8(align, _) => self.validate_store(cxt, *align, 8)?,
        I64Store16(align, _) => self.validate_store(cxt, *align, 16)?,
        I64Store32(align, _) => self.validate_store(cxt, *align, 32)?,

        MemorySize => {
          self.limits.first().ok_or(TypeError::TypeMismatch)?;
          cxt.push(ValueTypes::I32);
        }
        MemoryGrow => {
          self.limits.first().ok_or(TypeError::TypeMismatch)?;
          cxt.pop_i32()?;
          cxt.push(ValueTypes::I32);
        }

        I32CountLeadingZero => self.validate_unary(cxt)?,
        I32CountTrailingZero => self.validate_unary(cxt)?,
        I32CountNonZero => self.validate_unary(cxt)?,
        I64CountLeadingZero => self.validate_unary(cxt)?,
        I64CountTrailingZero => self.validate_unary(cxt)?,
        I64CountNonZero => self.validate_unary(cxt)?,

        I32Add => bin_op!(cxt),
        I32Sub => bin_op!(cxt),
        I32Mul => bin_op!(cxt),
        I32DivSign => bin_op!(cxt),
        I32DivUnsign => bin_op!(cxt),
        I32RemSign => bin_op!(cxt),
        I32RemUnsign => bin_op!(cxt),
        I32And => bin_op!(cxt),
        I32Or => bin_op!(cxt),
        I32Xor => bin_op!(cxt),
        I32ShiftLeft => bin_op!(cxt),
        I32ShiftRIghtSign => bin_op!(cxt),
        I32ShiftRightUnsign => bin_op!(cxt),
        I32RotateLeft => bin_op!(cxt),
        I32RotateRight => bin_op!(cxt),

        I64Add => bin_op!(cxt),
        I64Sub => bin_op!(cxt),
        I64Mul => bin_op!(cxt),
        I64DivSign => bin_op!(cxt),
        I64DivUnsign => bin_op!(cxt),
        I64RemSign => bin_op!(cxt),
        I64RemUnsign => bin_op!(cxt),
        I64And => bin_op!(cxt),
        I64Or => bin_op!(cxt),
        I64Xor => bin_op!(cxt),
        I64ShiftLeft => bin_op!(cxt),
        I64ShiftRightSign => bin_op!(cxt),
        I64ShiftRightUnsign => bin_op!(cxt),
        I64RotateLeft => bin_op!(cxt),
        I64RotateRight => bin_op!(cxt),

        I32EqualZero => test_op!(cxt),
        I64EqualZero => test_op!(cxt),

        Equal => rel_op!(cxt),
        NotEqual => rel_op!(cxt),
        LessThanSign => rel_op!(cxt),
        LessThanUnsign => rel_op!(cxt),
        I32GreaterThanSign => rel_op!(cxt),
        I32GreaterThanUnsign => rel_op!(cxt),
        I32LessEqualSign => rel_op!(cxt),
        I32LessEqualUnsign => rel_op!(cxt),
        I32GreaterEqualSign => rel_op!(cxt),
        I32GreaterEqualUnsign => rel_op!(cxt),

        I64Equal => rel_op!(cxt),
        I64NotEqual => rel_op!(cxt),
        I64LessThanSign => rel_op!(cxt),
        I64LessThanUnSign => rel_op!(cxt),
        I64GreaterThanSign => rel_op!(cxt),
        I64GreaterThanUnSign => rel_op!(cxt),
        I64LessEqualSign => rel_op!(cxt),
        I64LessEqualUnSign => rel_op!(cxt),
        I64GreaterEqualSign => rel_op!(cxt),
        I64GreaterEqualUnSign => rel_op!(cxt),

        F32Equal => rel_op!(cxt),
        F32NotEqual => rel_op!(cxt),
        F32LessThan => rel_op!(cxt),
        F32GreaterThan => rel_op!(cxt),
        F32LessEqual => rel_op!(cxt),
        F32GreaterEqual => rel_op!(cxt),

        F64Equal => rel_op!(cxt),
        F64NotEqual => rel_op!(cxt),
        F64LessThan => rel_op!(cxt),
        F64GreaterThan => rel_op!(cxt),
        F64LessEqual => rel_op!(cxt),
        F64GreaterEqual => rel_op!(cxt),

        F32Abs => self.validate_unary(cxt)?,
        F32Neg => self.validate_unary(cxt)?,
        F32Ceil => self.validate_unary(cxt)?,
        F32Floor => self.validate_unary(cxt)?,
        F32Trunc => self.validate_unary(cxt)?,
        F32Nearest => self.validate_unary(cxt)?,
        F32Sqrt => self.validate_unary(cxt)?,

        F32Add => bin_op!(cxt),
        F32Sub => bin_op!(cxt),
        F32Mul => bin_op!(cxt),
        F32Div => bin_op!(cxt),
        F32Min => bin_op!(cxt),
        F32Max => bin_op!(cxt),
        F32Copysign => bin_op!(cxt),

        F64Abs => self.validate_unary(cxt)?,
        F64Neg => self.validate_unary(cxt)?,
        F64Ceil => self.validate_unary(cxt)?,
        F64Floor => self.validate_unary(cxt)?,
        F64Trunc => self.validate_unary(cxt)?,
        F64Nearest => self.validate_unary(cxt)?,
        F64Sqrt => self.validate_unary(cxt)?,
        F64Add => bin_op!(cxt),
        F64Sub => bin_op!(cxt),
        F64Mul => bin_op!(cxt),
        F64Div => bin_op!(cxt),
        F64Min => bin_op!(cxt),
        F64Max => bin_op!(cxt),
        F64Copysign => bin_op!(cxt),

        Select => {
          cxt.pop_type()?;
          cxt.pop_type()?;
          let operand = cxt.pop_type()?;
          cxt.push(operand);
        }
        DropInst => {
          cxt.pop_type()?;
        }

        // To_convert_name_From
        // macro(cxt, from, to)
        I32WrapI64 => self.validate_convert(cxt, ValueTypes::I64, ValueTypes::I32)?,
        I32TruncSignF32 => self.validate_convert(cxt, ValueTypes::F32, ValueTypes::I32)?,
        I32TruncUnsignF32 => self.validate_convert(cxt, ValueTypes::F32, ValueTypes::I32)?,
        I32TruncSignF64 => self.validate_convert(cxt, ValueTypes::F64, ValueTypes::I32)?,
        I32TruncUnsignF64 => self.validate_convert(cxt, ValueTypes::F64, ValueTypes::I32)?,
        I64ExtendSignI32 => self.validate_convert(cxt, ValueTypes::I32, ValueTypes::I64)?,
        I64ExtendUnsignI32 => self.validate_convert(cxt, ValueTypes::I32, ValueTypes::I64)?,
        I64TruncSignF32 => self.validate_convert(cxt, ValueTypes::F32, ValueTypes::I64)?,
        I64TruncUnsignF32 => self.validate_convert(cxt, ValueTypes::F32, ValueTypes::I64)?,
        I64TruncSignF64 => self.validate_convert(cxt, ValueTypes::F64, ValueTypes::I64)?,
        I64TruncUnsignF64 => self.validate_convert(cxt, ValueTypes::F64, ValueTypes::I64)?,
        F32ConvertSignI32 => self.validate_convert(cxt, ValueTypes::I32, ValueTypes::F32)?,
        F32ConvertUnsignI32 => self.validate_convert(cxt, ValueTypes::I32, ValueTypes::F32)?,
        F32ConvertSignI64 => self.validate_convert(cxt, ValueTypes::I64, ValueTypes::F32)?,
        F32ConvertUnsignI64 => self.validate_convert(cxt, ValueTypes::I64, ValueTypes::F32)?,
        F32DemoteF64 => self.validate_convert(cxt, ValueTypes::F64, ValueTypes::F32)?,
        F64ConvertSignI32 => self.validate_convert(cxt, ValueTypes::I32, ValueTypes::F64)?,
        F64ConvertUnsignI32 => self.validate_convert(cxt, ValueTypes::I32, ValueTypes::F64)?,
        F64ConvertSignI64 => self.validate_convert(cxt, ValueTypes::I64, ValueTypes::F64)?,
        F64ConvertUnsignI64 => self.validate_convert(cxt, ValueTypes::I64, ValueTypes::F64)?,
        F64PromoteF32 => self.validate_convert(cxt, ValueTypes::F32, ValueTypes::F64)?,
        I32ReinterpretF32 => self.validate_convert(cxt, ValueTypes::F32, ValueTypes::I32)?,
        I64ReinterpretF64 => self.validate_convert(cxt, ValueTypes::F64, ValueTypes::I64)?,
        F32ReinterpretI32 => self.validate_convert(cxt, ValueTypes::I32, ValueTypes::F32)?,
        F64ReinterpretI64 => self.validate_convert(cxt, ValueTypes::I64, ValueTypes::F64)?,

        RuntimeValue(_) => unimplemented!(),
      }
    }
    Ok(())
  }

  pub fn validate(&self) -> Result<()> {
    // let grouped_imports = self.module.imports.group_by_kind();
    // let imports_function = grouped_imports.get(&FUNCTION_DESCRIPTOR)?;
    // let imports_table = grouped_imports.get(&TABLE_DESCRIPTOR)?;
    // let imports_memory = grouped_imports.get(&MEMORY_DESCRIPTOR)?;
    // let imports_global = grouped_imports.get(&GLOBAL_DESCRIPTOR)?;
    self.validate_elements()?;
    self.validate_function_types()?;
    self.validate_functions()?;

    // let global_instances =
    //   GlobalInstances::new_with_external(globals, &exports, &imports_global, &external_modules)?;

    // unimplemented!(
    //   "Type system(Also called as `validation`) not implemented yet.\n{:#?}",
    //   self.module
    // );
    Ok(())
  }
}
