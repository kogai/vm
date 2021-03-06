use super::decodable::{Decodable, Leb128Decodable, U32Decodable, U8Iterator};
use alloc::vec::Vec;
use core::convert::From;
use error::Result;
use function::FunctionType;
use isa::Isa;
use value_type::ValueTypes;

impl_decodable!(Section);
impl Leb128Decodable for Section {}
impl U32Decodable for Section {}

impl Decodable for Section {
  type Item = Vec<FunctionType>;
  fn decode(&mut self) -> Result<Self::Item> {
    let count_of_type = self.decode_leb128_u32()?;
    (0..count_of_type)
      .map(|_| {
        let mut parameters = vec![];
        let mut returns = vec![];
        let _type_function = Isa::from(self.next()?);
        let size_of_arity = self.decode_leb128_u32()?;
        for _ in 0..size_of_arity {
          parameters.push(ValueTypes::from(self.next()?));
        }
        let size_of_result = self.decode_leb128_u32()?;
        for _ in 0..size_of_result {
          returns.push(ValueTypes::from(self.next()?));
        }
        Ok(FunctionType::new(parameters, returns))
      })
      .collect::<Result<Vec<_>>>()
  }
}
