use decode::TableInstance;
use function::{FunctionInstance, FunctionType};
use global::GlobalInstance;
use memory::MemoryInstance;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::convert::From;
use std::default::Default;
use std::iter::Iterator;
use std::rc::Rc;
use store::Store;
use trap::{Result, Trap};

#[derive(Debug, Clone)]
pub enum ModuleDescriptor {
  Function(u32), // NOTE: Index of FunctionTypes
  Table(u32),
  Memory(u32),
  Global(u32),
}

impl From<(Option<u8>, u32)> for ModuleDescriptor {
  fn from(codes: (Option<u8>, u32)) -> Self {
    use self::ModuleDescriptor::*;
    match codes.0 {
      Some(0x0) => Function(codes.1),
      Some(0x1) => Table(codes.1),
      Some(0x2) => Memory(codes.1),
      Some(0x3) => Global(codes.1),
      x => unreachable!("Expected import descriptor, got {:?}", x),
    }
  }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum ModuleDescriptorKind {
  Function,
  Table,
  Memory,
  Global,
}

type ModuleName = Option<String>;
type Name = String;

#[derive(Debug, Clone)]
pub struct ExternalInterface {
  module_name: ModuleName,
  pub name: Name,
  pub descriptor: ModuleDescriptor,
}

impl ExternalInterface {
  pub fn new(module_name: ModuleName, name: Name, descriptor: ModuleDescriptor) -> Self {
    ExternalInterface {
      module_name,
      name,
      descriptor,
    }
  }
}

#[derive(Debug)]
pub struct ExternalInterfaces(HashMap<(ModuleName, Name), ExternalInterface>);

impl ExternalInterfaces {
  pub fn new() -> Self {
    ExternalInterfaces(HashMap::new())
  }

  pub fn insert(&mut self, value: ExternalInterface) {
    self
      .0
      .insert((value.module_name.clone(), value.name.clone()), value);
  }

  pub fn find_kind_by_idx(
    &self,
    idx: u32,
    kind: ModuleDescriptorKind,
  ) -> Option<&ExternalInterface> {
    self
      .0
      .iter()
      .find(
        |(_key, ExternalInterface { descriptor, .. })| match descriptor {
          ModuleDescriptor::Function(x) => ModuleDescriptorKind::Function == kind && *x == idx,
          ModuleDescriptor::Table(x) => ModuleDescriptorKind::Table == kind && *x == idx,
          ModuleDescriptor::Memory(x) => ModuleDescriptorKind::Memory == kind && *x == idx,
          ModuleDescriptor::Global(x) => ModuleDescriptorKind::Global == kind && *x == idx,
        },
      )
      .map(|(_, x)| x)
  }

  pub fn iter(&self) -> Iter<(ModuleName, Name), ExternalInterface> {
    self.0.iter()
  }

  pub fn group_by_kind(&self) -> HashMap<ModuleDescriptorKind, Self> {
    let mut buf_function = ExternalInterfaces::new();
    let mut buf_table = ExternalInterfaces::new();
    let mut buf_memory = ExternalInterfaces::new();
    let mut buf_global = ExternalInterfaces::new();
    let mut buf = HashMap::new();

    for (_module_name, x) in self.iter() {
      match x.descriptor {
        ModuleDescriptor::Function(_) => buf_function.insert(x.clone()),
        ModuleDescriptor::Table(_) => buf_table.insert(x.clone()),
        ModuleDescriptor::Memory(_) => buf_memory.insert(x.clone()),
        ModuleDescriptor::Global(_) => buf_global.insert(x.clone()),
      };
    }
    buf.insert(ModuleDescriptorKind::Function, buf_function);
    buf.insert(ModuleDescriptorKind::Table, buf_table);
    buf.insert(ModuleDescriptorKind::Memory, buf_memory);
    buf.insert(ModuleDescriptorKind::Global, buf_global);
    buf
  }
}

pub struct InternalModule {
  exports: ExternalInterfaces,
  imports: ExternalInterfaces,
}

impl InternalModule {
  pub fn new(exports: ExternalInterfaces, imports: ExternalInterfaces) -> Self {
    InternalModule { exports, imports }
  }

  pub fn get_export_by_key(&self, invoke: &str) -> Option<&ExternalInterface> {
    self.exports.0.get(&(None, invoke.to_owned()))
  }
}

#[derive(Debug, Clone)]
pub struct ExternalModule {
  pub function_instances: Vec<Rc<FunctionInstance>>,
  function_types: Vec<FunctionType>,
  memory_instances: Vec<MemoryInstance>,
  table_instances: Vec<TableInstance>,
  global_instances: Vec<GlobalInstance>,
}

impl ExternalModule {
  pub fn new(
    function_instances: Vec<Rc<FunctionInstance>>,
    function_types: Vec<FunctionType>,
    memory_instances: Vec<MemoryInstance>,
    table_instances: Vec<TableInstance>,
    global_instances: Vec<GlobalInstance>,
  ) -> Self {
    ExternalModule {
      function_instances,
      function_types,
      memory_instances,
      table_instances,
      global_instances,
    }
  }

  pub fn find_function_instance(
    &self,
    key: &ExternalInterface,
    function_types: &Vec<FunctionType>,
  ) -> Result<Rc<FunctionInstance>> {
    match key {
      ExternalInterface {
        descriptor: ModuleDescriptor::Function(idx),
        name,
        ..
      } => {
        let expected_type = function_types.get(*idx as usize)?;
        let instance = self
          .function_instances
          .iter()
          .find(|instance| instance.export_name == Some(name.to_owned()))
          .ok_or(Trap::UnknownImport)
          .map_err(|err| {
            if self
              .global_instances
              .iter()
              .find(|instance| instance.export_name == Some(name.to_owned()))
              .is_some()
              || self
                .memory_instances
                .iter()
                .find(|instance| instance.export_name == Some(name.to_owned()))
                .is_some()
              || self
                .table_instances
                .iter()
                .find(|instance| instance.export_name == Some(name.to_owned()))
                .is_some()
            {
              return Trap::IncompatibleImportType;
            };
            err
          })
          .map(|x| x.clone())?;

        instance
          .validate_type(expected_type)
          .map_err(|_| Trap::IncompatibleImportType)?;
        Ok(instance)
      }
      x => unreachable!("Expected function descriptor, got {:?}", x),
    }
  }
}

impl Default for ExternalModule {
  fn default() -> Self {
    ExternalModule {
      function_instances: vec![],
      function_types: vec![],
      memory_instances: vec![],
      table_instances: vec![],
      global_instances: vec![],
    }
  }
}

impl From<&Store> for ExternalModule {
  fn from(store: &Store) -> Self {
    ExternalModule {
      function_instances: store.function_instances.clone(),
      function_types: store.function_types.clone(),
      memory_instances: store.memory_instances.clone(),
      table_instances: store.table_instances.clone(),
      global_instances: store.global_instances.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExternalModules(HashMap<ModuleName, ExternalModule>);

impl ExternalModules {
  pub fn new() -> Self {
    ExternalModules(HashMap::new())
  }

  pub fn register_module(&mut self, key: ModuleName, value: ExternalModule) {
    self.0.insert(key, value);
  }

  pub fn get(&self, key: &ModuleName) -> Option<&ExternalModule> {
    self.0.get(key)
  }
}
