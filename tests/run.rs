#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
extern crate wasvm;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TypeValue {
  #[serde(rename = "type")]
  value_type: String,
  value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum Action {
  #[serde(rename = "invoke")]
  Invoke { field: String, args: Vec<TypeValue> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum TestCase {
  #[serde(rename = "module")]
  Module { line: usize, filename: String },
  #[serde(rename = "assert_return")]
  AssertReturn {
    line: usize,
    action: Action,
    expected: Vec<TypeValue>,
  },
  #[serde(rename = "assert_trap")]
  AssertTrap {
    line: usize,
    action: Action,
    text: String,
    expected: Vec<TypeValue>,
  },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestCases {
  source_filename: String,
  commands: Vec<TestCase>,
}

#[test]
fn run_all() {
  let mut buffer_json = vec![];
  let mut json = File::open("dist/i32.json").unwrap();
  json.read_to_end(&mut buffer_json).unwrap();
  let test_cases = serde_json::from_slice::<TestCases>(&buffer_json).unwrap();
  let (module, assertions) = test_cases.commands.split_first().unwrap();
  let wasm_file = if let TestCase::Module { line: _, filename } = module {
    let mut file = File::open(format!("dist/{}", filename)).unwrap();
    let mut tmp = [0; 8];
    let mut buffer = vec![];
    let _ = file.read_exact(&mut tmp).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    buffer
  } else {
    unreachable!();
  };

  for assertion in assertions {
    match assertion {
      TestCase::AssertReturn {
        line,
        action: Action::Invoke { field, args },
        expected,
      } => {
        println!("Testing spec at {}.", line);
        let mut vm = wasvm::Vm::new(wasm_file.clone());
        vm.run(
          field,
          args
            .iter()
            .map(|v| {
              let value = v.value.to_owned().unwrap();
              let value_type = v.value_type.to_owned();
              match value_type.as_ref() {
                "i32" => {
                  let actual_value = i32::from_str_radix(value.as_ref(), 10)
                    .expect(format!("Parameters must be {}", v.value_type).as_ref());
                  wasvm::byte::Values::I32(actual_value)
                }
                x => unimplemented!("{:?} is not implemented yet", x),
              }
            }).collect::<Vec<wasvm::byte::Values>>(),
        );
        let exp = expected.get(0).unwrap();
        assert_eq!(vm.get_result(), exp.value);
      }
      // TODO: Test AssertTrap
      _ => {
        println!("Skip assert_trap");
      }
    }
  }
}
