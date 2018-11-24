use std::ops::{BitAnd, BitOr, BitXor};
use trap::Trap;

#[derive(Debug, PartialEq, Clone)]
pub enum Values {
  I32(i32),
  I64(i64),
  // F32,
  // F64,
}

macro_rules! unary_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self) -> Self {
      match self {
        Values::I32(l) => Values::I32(l.$op()),
        Values::I64(l) => Values::I64(l.$op()),
      }
    }
  };
}

macro_rules! bynary_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Self {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) => Values::I32(l.$op(*r)),
        (Values::I64(l), Values::I64(r)) => Values::I64(l.$op(*r)),
        _ => unimplemented!(),
      }
    }
  };
}

macro_rules! bynary_try_inst {
  ($fn_name: ident,$op: ident) => {
    pub fn $fn_name(&self, other: &Self) -> Result<Self, Trap> {
      match (self, other) {
        (Values::I32(l), Values::I32(r)) =>  l.$op(*r).map(|n| Values::I32(n)) ,
        (Values::I64(l), Values::I64(r)) =>  l.$op(*r).map(|n| Values::I64(n)) ,
        _ => unimplemented!(),
      }
    }
  };
}

trait Arithmetic {
  fn equal_zero(&self) -> Self;
  fn count_leading_zero(&self) -> Self;
  fn count_trailing_zero(&self) -> Self;
  fn pop_count(&self) -> Self;

  fn less_than(&self, other: Self) -> Self;
  fn less_than_equal(&self, other: Self) -> Self;
  fn less_than_unsign(&self, other: Self) -> Self;
  fn less_than_equal_unsign(&self, other: Self) -> Self;

  fn greater_than(&self, other: Self) -> Self;
  fn greater_than_equal(&self, other: Self) -> Self;
  fn greater_than_unsign(&self, other: Self) -> Self;
  fn greater_than_equal_unsign(&self, other: Self) -> Self;

  fn equal(&self, other: Self) -> Self;
  fn not_equal(&self, other: Self) -> Self;

  fn shift_left(&self, other: Self) -> Self;
  fn shift_right_sign(&self, other: Self) -> Self;
  fn shift_right_unsign(&self, other: Self) -> Self;

  fn wasm_rotate_left(&self, other: Self) -> Self;
  fn wasm_rotate_right(&self, other: Self) -> Self;

  fn rem_s(&self, other: Self) -> Result<Self, Trap>
  where
    Self: Sized;
  fn rem_u(&self, other: Self) -> Result<Self, Trap>
  where
    Self: Sized;
  fn div_s(&self, other: Self) -> Result<Self, Trap>
  where
    Self: Sized;
  fn div_u(&self, other: Self) -> Result<Self, Trap>
  where
    Self: Sized;
}

macro_rules! impl_traits {
  ($ty: ty, $unsign: ty) => {
    impl Arithmetic for $ty {
      fn equal_zero(&self) -> Self {
        if self == &0 {
          1
        } else {
          0
        }
      }
      fn count_leading_zero(&self) -> Self {
        self.leading_zeros() as $ty
      }
      fn count_trailing_zero(&self) -> Self {
        self.trailing_zeros() as $ty
      }
      fn pop_count(&self) -> Self {
        self.count_ones() as $ty
      }

      fn less_than(&self, other: Self) -> Self {
        if self.lt(&other) {
          1
        } else {
          0
        }
      }
      fn less_than_equal(&self, other: Self) -> Self {
        if self.le(&other) {
          1
        } else {
          0
        }
      }
      fn less_than_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        if l1.lt(&r1) {
          1
        } else {
          0
        }
      }
      fn less_than_equal_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        if l1.le(&r1) {
          1
        } else {
          0
        }
      }
      fn greater_than(&self, other: Self) -> Self {
        if self.gt(&other) {
          1
        } else {
          0
        }
      }
      fn greater_than_equal(&self, other: Self) -> Self {
        if self.ge(&other) {
          1
        } else {
          0
        }
      }
      fn greater_than_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        let result = l1.gt(&r1);
        if result {
          1
        } else {
          0
        }
      }

      fn greater_than_equal_unsign(&self, other: Self) -> Self {
        let l1 = *self as $unsign;
        let r1 = other as $unsign;
        let result = l1.ge(&r1);
        if result {
          1
        } else {
          0
        }
      }
      fn equal(&self, other: Self) -> Self {
        if self.eq(&other) {
          1
        } else {
          0
        }
      }
      fn not_equal(&self, other: Self) -> Self {
        if self.ne(&other) {
          1
        } else {
          0
        }
      }
      fn shift_left(&self, other: Self) -> Self {
        self.wrapping_shl(other as u32)
      }
      fn shift_right_sign(&self, other: Self) -> Self {
        let shifted = self.wrapping_shr(other as u32);
        let casted = (shifted as $unsign) as $ty;
        casted
      }
      fn shift_right_unsign(&self, other: Self) -> Self {
        let i1 = *self as $unsign;
        let shifted = i1.wrapping_shr(other as u32) as $ty;
        shifted
      }

      fn wasm_rotate_left(&self, other: Self) -> Self {
        self.rotate_left(other as u32)
      }

      fn wasm_rotate_right(&self, other: Self) -> Self {
        self.rotate_right(other as u32)
      }

      fn rem_s(&self, other: Self) -> Result<Self, Trap> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, _) = self.overflowing_rem(other);
        Ok(divined)
      }

      fn rem_u(&self, other: Self) -> Result<Self, Trap> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*self as $unsign).overflowing_rem(other as $unsign);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(divined as $ty)
        }
      }
      fn div_u(&self, other: Self) -> Result<Self, Trap> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = (*self as $unsign).overflowing_div(other as $unsign);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(divined as $ty)
        }
      }
      fn div_s(&self, other: Self) -> Result<Self, Trap> {
        if other == 0 {
          return Err(Trap::DivisionByZero);
        }
        let (divined, overflowed) = self.overflowing_div(other);
        if overflowed {
          Err(Trap::DivisionOverflow)
        } else {
          Ok(divined)
        }
      }
    }
  };
}

impl_traits!(i32, u32);
impl_traits!(i64, u64);

impl Values {
  bynary_inst!(and, bitand);
  bynary_inst!(or, bitor);
  bynary_inst!(xor, bitxor);
  bynary_inst!(add, wrapping_add);
  bynary_inst!(sub, wrapping_sub);
  bynary_inst!(mul, wrapping_mul);

  bynary_inst!(less_than, less_than);
  bynary_inst!(less_than_equal, less_than_equal);
  bynary_inst!(less_than_unsign, less_than_unsign);
  bynary_inst!(less_than_equal_unsign, less_than_equal_unsign);

  bynary_inst!(greater_than, greater_than);
  bynary_inst!(greater_than_equal, greater_than_equal);
  bynary_inst!(greater_than_unsign, greater_than_unsign);
  bynary_inst!(greater_than_equal_unsign, greater_than_equal_unsign);
  bynary_inst!(equal, equal);
  bynary_inst!(not_equal, not_equal);

  bynary_inst!(shift_left, shift_left);
  bynary_inst!(shift_right_sign, shift_right_sign);
  bynary_inst!(shift_right_unsign, shift_right_unsign);
  bynary_inst!(wasm_rotate_left, wasm_rotate_left);
  bynary_inst!(wasm_rotate_right, wasm_rotate_right);

  bynary_try_inst!(rem_s, rem_s);
  bynary_try_inst!(rem_u, rem_u);
  bynary_try_inst!(div_s, div_s);
  bynary_try_inst!(div_u, div_u);

  unary_inst!(equal_zero, equal_zero);
  unary_inst!(count_leading_zero, count_leading_zero);
  unary_inst!(count_trailing_zero, count_trailing_zero);
  unary_inst!(pop_count, pop_count);

  pub fn is_truthy(&self) -> bool {
    match &self {
      Values::I32(n) => *n > 0,
      _ => unimplemented!(),
    }
  }

  pub fn extend_to_i64(&self) -> Self {
    match self {
      Values::I32(l) => Values::I64(i64::from(*l)),
      _ => unimplemented!(),
    }
  }
}
