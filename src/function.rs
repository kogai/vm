use inst::Inst;
use std::fmt;
use std::rc::Rc;
use trap::Result;
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

  pub fn get_return_count(&self) -> u32 {
    self.returns.len() as u32
  }

  #[allow(dead_code)]
  pub fn get_parameter_types<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.parameters
  }

  pub fn get_return_types<'a>(&'a self) -> &'a Vec<ValueTypes> {
    &self.returns
  }
}

impl FunctionType {
  pub fn get_arity(&self) -> u32 {
    self.parameters.len() as u32
  }
}

#[derive(PartialEq, Clone)]
pub struct FunctionInstance {
  export_name: Option<String>,
  function_type: Result<FunctionType>,
  pub locals: Vec<ValueTypes>,
  type_idex: u32,
  body: Rc<Vec<Inst>>,
}

impl fmt::Debug for FunctionInstance {
  // TODO: Consider also to present instructions.
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let name = match self.export_name {
      Some(ref n) => n,
      _ => "_",
    };
    let function_type = match self.function_type {
      Ok(ref f) => format!("{:?}", f),
      Err(ref err) => format!("{:?}", err),
    };
    write!(f, "[{}] {}: {}", self.type_idex, name, function_type)
  }
}

impl FunctionInstance {
  pub fn new(
    export_name: Option<String>,
    function_type: Result<FunctionType>,
    locals: Vec<ValueTypes>,
    type_idex: u32,
    body: Vec<Inst>,
  ) -> Rc<Self> {
    Rc::new(FunctionInstance {
      export_name,
      function_type,
      locals,
      type_idex,
      body: Rc::new(body),
    })
  }

  pub fn get(&self, idx: usize) -> Option<&Inst> {
    self.body.get(idx)
  }

  pub fn get_expressions_count(&self) -> usize {
    self.body.len()
  }

  pub fn get_arity(&self) -> u32 {
    match self.function_type {
      Ok(ref f) => f.get_arity(),
      _ => 0,
    }
  }

  pub fn get_function_type(&self) -> Result<FunctionType> {
    self.function_type.to_owned()
  }

  pub fn find(&self, key: &str) -> bool {
    // FIXME: When using function_index, we might get exported function by O(1).
    match &self.export_name {
      Some(name) => name.as_str() == key,
      _ => false,
    }
  }
}
