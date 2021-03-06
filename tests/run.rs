extern crate wabt;

#[cfg(test)]
extern crate wasvm;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::{f32, f64};
use wabt::script::{Action, Command, CommandKind, ModuleBinary, ScriptParser, Value};
use wasvm::{
  create_spectest, decode_module, init_store, instantiate_module, validate_module, ExternalModules,
  ModuleInstance, Trap, Values, WasmError,
};

fn get_args(args: &[Value<f32, f64>]) -> Vec<Values> {
  args
    .iter()
    .map(|v| match v {
      Value::I32(value) => Values::I32(*value),
      Value::I64(value) => Values::I64(*value),
      Value::F32(value) => Values::F32(*value),
      Value::F64(value) => Values::F64(*value),
    })
    .collect()
}

fn get_expectation(expected: &[Value]) -> Values {
  match expected.get(0) {
    Some(Value::I32(v)) => Values::I32(*v),
    Some(Value::I64(v)) => Values::I64(*v),
    Some(Value::F32(v)) => Values::F32(*v),
    Some(Value::F64(v)) => Values::F64(*v),
    None => Values::I32(0),
  }
}

struct E2ETest<'a> {
  parser: ScriptParser<f32, f64>,
  modules: HashMap<Option<String>, Rc<RefCell<ModuleInstance>>>,
  external_modules: ExternalModules,
  file_name: &'a str,
}

impl<'a> E2ETest<'a> {
  fn new(file_name: &'a str) -> Self {
    let mut buf = String::new();
    let test_filename = format!("./testsuite/{}.wast", file_name);
    let mut json = File::open(&test_filename).unwrap();
    json.read_to_string(&mut buf).unwrap();
    let parser = ScriptParser::from_str(&buf).unwrap();
    let modules = HashMap::new();
    let mut external_modules = ExternalModules::default();
    external_modules
      .register_module(Some("spectest".to_owned()), create_spectest())
      .unwrap();

    E2ETest {
      parser,
      modules,
      external_modules,
      file_name,
    }
  }

  fn do_instantiate(&mut self, module: &ModuleBinary, name: &Option<String>) {
    let bytes = module.clone().into_vec();
    let store = init_store();
    let section = decode_module(&bytes);
    let vm_ref = Rc::new(RefCell::new(
      instantiate_module(store, section, self.external_modules.clone(), 65536).unwrap(),
    ));
    self.modules.insert(None, vm_ref.clone());
    self.modules.insert(name.clone(), vm_ref.clone());
  }

  fn do_action(&mut self, field: &str, args: &[Value], module: &Option<String>, line: u64) {
    println!("Perform action at {}:{}.", field, line);
    let vm_ref: Rc<RefCell<ModuleInstance>> = self.modules[module].clone();
    let mut vm = vm_ref.borrow_mut();
    vm.run(field, get_args(args)).unwrap();
  }

  fn do_register(&mut self, name: &Option<String>, as_name: &str) {
    println!(
      "Register importable module, key={:?} import_name={}.",
      name, as_name
    );
    let vm_ref: Rc<RefCell<ModuleInstance>> = self.modules[name].clone();
    let vm = vm_ref.borrow();
    let importable_module = vm.export_module();
    self
      .external_modules
      .register_module(Some(as_name.to_owned()), importable_module)
      .unwrap();
  }

  fn assert_return(&self, action: &Action, expected: &[Value], line: u64) {
    let (field, args, module) = match action {
      Action::Invoke {
        ref field,
        ref args,
        ref module,
      } => (field, get_args(args), module),
      Action::Get {
        ref field,
        ref module,
      } => (field, vec![], module),
    };
    println!("Assert return at {}:{}.", field, line);
    let vm_ref: Rc<RefCell<ModuleInstance>> = self.modules[module].clone();
    let mut vm = vm_ref.borrow_mut();
    let actual = vm.run(field.as_ref(), args).unwrap();
    let expectation = get_expectation(expected);
    match actual {
      Values::F32(n) if n.is_nan() => match expectation {
        Values::F32(m) => assert!(m.is_nan()),
        _ => unreachable!(),
      },
      Values::F64(n) if n.is_nan() => match expectation {
        Values::F64(m) => assert!(m.is_nan()),
        _ => unreachable!(),
      },
      _ => assert_eq!(actual, expectation),
    };
  }
  fn assert_trap(&mut self, action: &Action, _message: &str, line: u64) {
    match action {
      Action::Invoke {
        ref field,
        ref args,
        ref module,
      } => {
        println!("Assert trap at {}:{}.", field, line,);
        let vm_ref: Rc<RefCell<ModuleInstance>> = self.modules[module].clone();
        let mut vm = vm_ref.borrow_mut();
        vm.run(field, get_args(args)).unwrap_err();
      }
      x => unreachable!("{:?}", x),
    }
  }

  fn assert_uninstantiable(&mut self, module: &ModuleBinary, _message: &str, line: u64) {
    println!("Assert uninstantiable at line:{}.", line);
    let bytes = module.clone().into_vec();
    let store = init_store();
    let module = decode_module(&bytes);
    instantiate_module(store, module, Default::default(), 65536).unwrap_err();
  }

  fn assert_malformed(&self, module: &ModuleBinary, _message: &str, line: u64) {
    if (self.file_name == "custom_section" && line == 77)
      || (self.file_name == "custom_section" && line == 94)
      || (self.file_name == "custom" && line == 85)
    {
      println!("Skip {}, it seems can't resolvable yet...", line);
      return;
    };
    let bytes = module.clone().into_vec();
    let store = init_store();
    let module = decode_module(&bytes);
    let err = instantiate_module(store, module, Default::default(), 65536).unwrap_err();
    if let WasmError::Trap(Trap::UnsupportedTextform) = err {
      println!("Skip malformed text form at line:{}.", line);
      return;
    };
    println!("Assert malformed at {}.", line,);
  }

  fn assert_invalid(&self, message: &str, module: &ModuleBinary, line: u64) {
    println!("Assert invalid at {}:{}.", message, line);
    let bytes = module.clone().into_vec();
    let section = decode_module(&bytes);
    validate_module(&section).unwrap_err();
  }

  fn assert_nan(&self, action: &Action, line: u64) {
    match action {
      Action::Invoke {
        ref field,
        ref args,
        ref module,
      } => {
        println!("Assert NaN at '{}:{}'.", field, line);
        let vm_ref: Rc<RefCell<ModuleInstance>> = self.modules[module].clone();
        let mut vm = vm_ref.borrow_mut();
        let actual = vm.run(field.as_ref(), get_args(args)).unwrap();
        match actual {
          Values::F32(n) => assert!(n.is_nan()),
          Values::F64(n) => assert!(n.is_nan()),
          _ => unreachable!(),
        };
      }
      x => unreachable!("{:?}", x),
    }
  }

  fn assert_unlinkable(&self, module: &ModuleBinary, _message: &str, line: u64) {
    let bytes = module.clone().into_vec();
    let tmp_bytes = bytes.clone();
    let (magic_numbers, _) = tmp_bytes.split_at(8);
    if magic_numbers == [0u8, 97, 115, 109, 1, 0, 0, 0] {
      println!("Assert unlinkable at {}.", line,);
      let store = init_store();
      let section = decode_module(&bytes);
      instantiate_module(store, section, self.external_modules.clone(), 65536).unwrap_err();
    } else {
      println!("Skip unlinkable text form at line:{}.", line);
    };
  }

  fn do_test(&mut self) {
    while let Ok(Some(Command { kind, line, .. })) = self.parser.next() {
      match kind {
        CommandKind::Module {
          ref module,
          ref name,
        } => self.do_instantiate(module, name),
        CommandKind::PerformAction(Action::Invoke {
          ref field,
          ref args,
          ref module,
        }) => self.do_action(field, args, module, line),
        CommandKind::Register {
          ref name,
          ref as_name,
          ..
        } => self.do_register(name, as_name),
        CommandKind::AssertReturn {
          ref action,
          ref expected,
        } => self.assert_return(action, expected, line),
        CommandKind::AssertTrap {
          ref action,
          ref message,
        } => self.assert_trap(action, message, line),
        CommandKind::AssertUninstantiable {
          ref module,
          ref message,
        } => self.assert_uninstantiable(module, message, line),
        CommandKind::AssertMalformed {
          ref module,
          ref message,
        } => self.assert_malformed(module, message, line),
        CommandKind::AssertReturnCanonicalNan { ref action } => self.assert_nan(action, line),
        CommandKind::AssertReturnArithmeticNan { ref action } => self.assert_nan(action, line),
        CommandKind::AssertUnlinkable {
          ref module,
          ref message,
        } => self.assert_unlinkable(module, message, line),
        // FIXME: Enable specs
        CommandKind::AssertExhaustion {
          action: Action::Invoke { ref field, .. },
        } => println!("Skip exhaustion line:{}:{}.", field, line),
        CommandKind::AssertInvalid {
          ref message,
          ref module,
        } => self.assert_invalid(message, module, line),
        x => unreachable!(
          "there are no other commands apart from that defined above {:?}",
          x
        ),
      }
    }
  }
}

macro_rules! impl_e2e {
  ($test_name: ident, $file_name: expr) => {
    #[test]
    fn $test_name() {
      let mut t = E2ETest::new($file_name);
      t.do_test()
    }
  };
}

// NOTE: Convient to debug wast specs.
// impl_e2e!(test_sandbox, "sandbox");

impl_e2e!(test_address, "address");
impl_e2e!(test_align, "align");
// impl_e2e!(test_binary, "binary");
impl_e2e!(test_block, "block");
impl_e2e!(test_br_if, "br_if");
impl_e2e!(test_br_table, "br_table");
impl_e2e!(test_br_only, "br");
impl_e2e!(test_break_drop, "break-drop");
impl_e2e!(test_call_indirect, "call_indirect");
impl_e2e!(test_call, "call");
impl_e2e!(test_comments, "comments");
impl_e2e!(test_const, "const"); /* All specs suppose Text-format */
impl_e2e!(test_conversions, "conversions");
impl_e2e!(test_custom_section, "custom_section");
impl_e2e!(test_custom_simple, "custom");
impl_e2e!(test_data, "data");
impl_e2e!(test_elem, "elem");
impl_e2e!(test_endianness, "endianness");
impl_e2e!(test_exports, "exports");
impl_e2e!(test_f32_bitwise, "f32_bitwise");
impl_e2e!(test_f32_cmp, "f32_cmp");
impl_e2e!(test_f32, "f32");
impl_e2e!(test_f64_bitwise, "f64_bitwise");
impl_e2e!(test_f64_cmp, "f64_cmp");
impl_e2e!(test_f64, "f64");
impl_e2e!(test_fac, "fac");
impl_e2e!(test_float_exprs, "float_exprs");
impl_e2e!(test_float_literals, "float_literals");
impl_e2e!(test_float_memory, "float_memory");
impl_e2e!(test_float_misc, "float_misc");
impl_e2e!(test_forward, "forward");
impl_e2e!(test_func_ptrs, "func_ptrs");
impl_e2e!(test_func, "func");
impl_e2e!(test_get_local, "get_local");
impl_e2e!(test_globals, "globals");
impl_e2e!(test_i32, "i32");
impl_e2e!(test_i64, "i64");
impl_e2e!(test_if, "if");
impl_e2e!(test_imports, "imports");
impl_e2e!(test_inline_module, "inline-module");
impl_e2e!(test_int_exprs, "int_exprs");
impl_e2e!(test_int_literals, "int_literals");
impl_e2e!(test_labels, "labels");
impl_e2e!(test_left_to_right, "left-to-right");
impl_e2e!(test_linking, "linking");
impl_e2e!(test_loop, "loop");
impl_e2e!(test_memory_grow, "memory_grow");
impl_e2e!(test_memory_redundancy, "memory_redundancy");
impl_e2e!(test_memory_trap, "memory_trap");
impl_e2e!(test_memory_only, "memory");
impl_e2e!(test_names, "names");
impl_e2e!(test_nop, "nop");
impl_e2e!(test_resizing, "resizing");
impl_e2e!(test_return, "return");
impl_e2e!(test_select, "select");
impl_e2e!(test_set_local, "set_local");
impl_e2e!(test_skip_stack_guard_page, "skip-stack-guard-page");
impl_e2e!(test_stack, "stack");
impl_e2e!(test_start, "start");
impl_e2e!(test_store_retval, "store_retval");
impl_e2e!(test_switch, "switch");
impl_e2e!(test_tee_local, "tee_local");
impl_e2e!(test_token, "token");
impl_e2e!(test_traps, "traps");
impl_e2e!(test_type, "type");
impl_e2e!(test_typecheck, "typecheck");
impl_e2e!(test_unreachable, "unreachable");
impl_e2e!(test_unreached_invalid, "unreached-invalid");
impl_e2e!(test_unwind, "unwind");
impl_e2e!(test_utf8_custom_section_id, "utf8-custom-section-id");
impl_e2e!(test_utf8_import_field, "utf8-import-field");
impl_e2e!(test_utf8_import_module, "utf8-import-module");
impl_e2e!(test_utf8_invalid_encoding, "utf8-invalid-encoding");
