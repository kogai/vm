use code::ValueTypes;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use inst::{Instructions, TypeKind};
use memory::MemoryInstance;
use store::Store;
use table::TableInstance;
use trap::{Result, Trap};

pub struct Context {
  function_instances: Vec<FunctionInstance>,
  memory_instances: Vec<MemoryInstance>,
  table_instances: Vec<TableInstance>,
  global_instances: Vec<GlobalInstance>,
  _type_context: Vec<TypeKind>,
}

impl Context {
  pub fn new(
    function_instances: Vec<FunctionInstance>,
    memory_instances: Vec<MemoryInstance>,
    table_instances: Vec<TableInstance>,
    global_instances: Vec<GlobalInstance>,
  ) -> Self {
    Context {
      function_instances,
      memory_instances,
      table_instances,
      global_instances,
      _type_context: vec![],
    }
  }

  pub fn without_validate(self) -> Result<Store> {
    Ok(Store::new(
      self.function_instances,
      self.memory_instances,
      self.table_instances,
      self.global_instances,
    ))
  }

  #[allow(dead_code)]
  pub fn validate(self) -> Result<Store> {
    // FIXME: Suppress compile type validate until ready.
    self
      .function_instances
      .iter()
      .map(|function_instance| {
        let function_type = function_instance.get_function_type().to_owned()?;
        let expect_return_type = function_type.get_return_types();
        let actual_return_type = self.reduction_instructions(function_instance, &function_type)?;
        if expect_return_type != &actual_return_type {
          return Err(Trap::TypeMismatch);
        }
        Ok(())
      })
      .collect::<Result<Vec<_>>>()?;
    Ok(Store::new(
      self.function_instances,
      self.memory_instances,
      self.table_instances,
      self.global_instances,
    ))
  }

  fn reduction_instructions(
    &self,
    function_instance: &FunctionInstance,
    function_type: &FunctionType,
  ) -> Result<Vec<ValueTypes>> {
    let (expressions, mut locals) = function_instance.call();
    let mut parameters = function_type.get_parameter_types().to_owned();
    parameters.append(&mut locals);
    let mut instructions = Instructions::new(
      expressions,
      vec![0],
      self
        .function_instances
        .iter()
        .map(|f| f.function_type.to_owned())
        .collect(),
    );
    let return_type = self.reduction_instructions_internal(&mut instructions, &parameters)?;
    Ok(vec![return_type])
  }
  // NOTE: Currently, WASM specification supposes to single return value.
  fn reduction_instructions_internal(
    &self,
    instructions: &mut Instructions,
    _locals: &Vec<ValueTypes>,
  ) -> Result<ValueTypes> {
    let mut _return_type: ValueTypes;
    while !instructions.is_next_end_or_else() {
      let instruction = instructions.pop_ref()?;
      match instruction.into() {
        TypeKind::Canonical(_ty) => {
          println!("instruction={:?}", instruction);
          unimplemented!();
        }
        TypeKind::Polymophic => {
          println!("instruction={:?}", instruction);
          unimplemented!();
        }
        TypeKind::Void => {}
      }
    }
    unimplemented!();
  }
}

/*
#[cfg(test)]
mod tests {
  use super::*;
  use code::ValueTypes;
  use function::FunctionType;
  use inst::Inst;

  #[test]
  fn test_validate_return_type() {
    let export_name = None;
    let function_type = Ok(FunctionType::new(vec![], vec![ValueTypes::I32]));
    let locals = vec![];
    let type_idx = 0;
    let body = vec![Inst::I64Const(0), Inst::End];
    let actual = Context::new(
      vec![FunctionInstance::new(
        export_name,
        function_type,
        locals,
        type_idx,
        body,
      )],
      vec![],
      vec![],
      vec![],
    )
    .validate();
    assert_eq!(actual.unwrap_err(), Trap::TypeMismatch);
  }
  #[test]
  fn test_validate_return_if() {
    use self::Inst::*;
    let export_name = None;
    let function_type = Ok(FunctionType::new(vec![], vec![ValueTypes::I32]));
    let locals = vec![];
    let type_idx = 0;
    let body = vec![
      If(0, 0),
      RuntimeValue(ValueTypes::I32),
      I64Const(0),
      Else,
      I64Const(0),
      If(0, 0),
      RuntimeValue(ValueTypes::I32),
      F32Const(0.0),
      Else,
      F32Const(0.0),
      End,
      End,
      End,
    ];
    let actual = Context::new(
      vec![FunctionInstance::new(
        export_name,
        function_type,
        locals,
        type_idx,
        body,
      )],
      vec![],
      vec![],
      vec![],
    )
    .validate();
    assert_eq!(actual.unwrap_err(), Trap::TypeMismatch);
  }
}
*/