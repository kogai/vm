use alloc::prelude::*;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use inst::Inst;
use trap::{Result, Trap};
use value_type::ValueTypes;

#[derive(PartialEq, Clone)]
pub struct FunctionType {
  parameters: Vec<ValueTypes>,
  returns: Vec<ValueTypes>,
}

impl fmt::Debug for FunctionType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "({}) -> ({})",
      self
        .parameters
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
      self
        .returns
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<String>>()
        .join(", "),
    )
  }
}

impl FunctionType {
  pub fn new(parameters: Vec<ValueTypes>, returns: Vec<ValueTypes>) -> Self {
    FunctionType {
      parameters,
      returns,
    }
  }

  #[allow(dead_code)]
  pub fn get_parameter_types<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.parameters
  }

  pub fn get_return_types<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.returns
  }

  pub fn get_arity(&self) -> u32 {
    self.parameters.len() as u32
  }
}

#[derive(PartialEq)]
pub struct FunctionInstance {
  pub export_name: Option<String>,
  pub(crate) function_type: FunctionType,
  pub locals: Vec<ValueTypes>,
  body: Vec<Inst>,
}

impl fmt::Debug for FunctionInstance {
  // TODO: Consider also to present instructions.
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let name = match self.export_name {
      Some(ref n) => n,
      _ => "_",
    };
    f.debug_struct("FunctionInstance")
      .field("export_name", &name)
      .field("function_type", &self.function_type)
      .field("instructions", &format_args!("{:?}", self.body))
      .finish()
  }
}

impl FunctionInstance {
  pub fn new(
    export_name: Option<String>,
    function_type: FunctionType,
    locals: Vec<ValueTypes>,
    body: Vec<Inst>,
  ) -> Rc<Self> {
    Rc::new(FunctionInstance {
      export_name,
      function_type,
      locals,
      body,
    })
  }

  pub fn get(&self, idx: usize) -> Option<&Inst> {
    self.body.get(idx)
  }

  pub fn get_expressions_count(&self) -> usize {
    self.body.len()
  }

  pub fn get_arity(&self) -> u32 {
    self.function_type.parameters.len() as u32
  }

  pub fn get_function_type(&self) -> FunctionType {
    self.function_type.to_owned()
  }

  pub fn get_return_type(&self) -> &Vec<ValueTypes> {
    &self.function_type.returns
  }

  pub fn get_return_count(&self) -> u32 {
    self.function_type.returns.len() as u32
  }

  pub fn validate_type(&self, other: &FunctionType) -> Result<()> {
    if &self.function_type != other {
      return Err(Trap::TypeMismatch);
    }
    Ok(())
  }
}
