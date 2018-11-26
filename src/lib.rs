#![feature(try_trait)]
mod byte;
mod code;
mod function;
mod inst;
mod memory;
mod stack;
mod store;
mod trap;
pub mod value;

use inst::Inst;
use stack::Frame;
use stack::{Stack, StackEntry};
use store::Store;
use trap::{Result, Trap};
use value::Values;

macro_rules! impl_load_inst {
    ($load_data_width: expr, $self: ident, $offset: ident, $value_kind: expr) => {{
        let width = $load_data_width / 8;
        let i = match $self.stack.pop_value() {
            Values::I32(i) => i,
            x => unreachable!("{:?}", x),
        } as u32;
        let (ea, overflowed) = i.overflowing_add(*$offset); // NOTE: What 'ea' stands for?
        if overflowed {
            return Err(Trap::MemoryAccessOutOfBounds);
        };
        let (ptr, overflowed) = ea.overflowing_add(width);
        if overflowed || $self.store.data_size_small_than(ptr) {
            return Err(Trap::MemoryAccessOutOfBounds);
        };
        let data = $self.store.load_data(ea, ptr, $value_kind);
        $self.stack.push(StackEntry::new_value(data));
    }};
}

macro_rules! impl_unary_inst {
    ($self: ident, $op: ident) => {{
        let popped = $self.stack.pop_value();
        let value = popped.$op();
        $self.stack.push(StackEntry::new_value(value));
    }};
}

macro_rules! impl_binary_inst {
    ($self: ident, $op: ident) => {{
        let right = $self.stack.pop_value();
        let left = $self.stack.pop_value();
        let value = left.$op(&right);
        $self.stack.push(StackEntry::new_value(value));
    }};
}

macro_rules! impl_try_binary_inst {
    ($self: ident, $op: ident) => {{
        let right = $self.stack.pop_value();
        let left = $self.stack.pop_value();
        let value = left.$op(&right);
        match value {
            Ok(result) => {
                $self.stack.push(StackEntry::new_value(result));
            }
            Err(trap) => {
                return Err(trap);
            }
        }
    }};
}

pub struct Vm {
    store: Store,
    stack: Stack,
}

impl Vm {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        let mut bytes = byte::Byte::new(bytes);
        match bytes.decode() {
            Ok(store) => Ok(Vm {
                store,
                stack: Stack::new(65536),
            }),
            Err(err) => Err(err),
        }
    }

    fn evaluate_instructions(&mut self, expressions: &Vec<Inst>) -> Result<()> {
        use self::Inst::*;
        for expression in expressions.iter() {
            match expression {
                GetLocal(idx) => {
                    let frame_ptr = self.stack.get_frame_ptr();
                    let value = self.stack.get(*idx + frame_ptr)?;
                    self.stack.push(value);
                }
                SetLocal(idx) => {
                    let value = self.stack.pop().map(|s| s.to_owned())?;
                    let frame_ptr = self.stack.get_frame_ptr();
                    self.stack.set(*idx + frame_ptr, value);
                }
                TeeLocal(idx) => {
                    let value = self.stack.pop().map(|s| s.to_owned())?;
                    self.stack.push(value.clone());
                    let frame_ptr = self.stack.get_frame_ptr();
                    self.stack.set(*idx + frame_ptr, value);
                }
                Call(idx) => {
                    let operand = self.stack.pop_value();
                    self.call(*idx, vec![operand]);
                    let _ = self.evaluate();
                }
                I32Add | I64Add | F32Add => impl_binary_inst!(self, add),
                I32Sub | I64Sub | F32Sub => impl_binary_inst!(self, sub),
                I32Mul | I64Mul | F32Mul => impl_binary_inst!(self, mul),
                I32DivUnsign | I64DivUnsign => impl_try_binary_inst!(self, div_u),
                I32DivSign | I64DivSign => impl_try_binary_inst!(self, div_s),
                F32Div => impl_binary_inst!(self, div_f),
                I32RemSign | I64RemSign => impl_try_binary_inst!(self, rem_s),
                I32RemUnsign | I64RemUnsign => impl_try_binary_inst!(self, rem_u),
                F32Min => impl_binary_inst!(self, min),
                F32Max => impl_binary_inst!(self, max),
                F32Sqrt => impl_unary_inst!(self, sqrt),
                F32Ceil => impl_unary_inst!(self, ceil),
                F32Floor => impl_unary_inst!(self, floor),
                F32Trunc => impl_unary_inst!(self, trunc),
                F32Nearest => impl_unary_inst!(self, nearest),
                I32Const(n) => self.stack.push(StackEntry::new_value(Values::I32(*n))),
                I64Const(n) => self.stack.push(StackEntry::new_value(Values::I64(*n))),
                Select => {
                    let cond = &self.stack.pop_value();
                    let false_br = self.stack.pop_value();
                    let true_br = self.stack.pop_value();
                    if cond.is_truthy() {
                        self.stack.push(StackEntry::new_value(true_br));
                    } else {
                        self.stack.push(StackEntry::new_value(false_br));
                    }
                }
                DropInst => {
                    self.stack.pop_value();
                }
                LessThanSign | I64LessThanSign => impl_binary_inst!(self, less_than),
                LessThanUnsign | I64LessThanUnSign => impl_binary_inst!(self, less_than_unsign),
                I32LessEqualSign | I64LessEqualSign => impl_binary_inst!(self, less_than_equal),
                I32LessEqualUnsign | I64LessEqualUnSign => {
                    impl_binary_inst!(self, less_than_equal_unsign)
                }
                I32GreaterEqualSign | I64GreaterEqualSign => {
                    impl_binary_inst!(self, greater_than_equal)
                }
                I32GreaterThanSign | I64GreaterThanSign => impl_binary_inst!(self, greater_than),
                I32GreaterThanUnsign | I64GreaterThanUnSign => {
                    impl_binary_inst!(self, greater_than_unsign)
                }
                I32GreaterEqualUnsign | I64GreaterEqualUnSign => {
                    impl_binary_inst!(self, greater_than_equal_unsign)
                }
                Equal | I64Equal => impl_binary_inst!(self, equal),
                NotEqual | I64NotEqual => impl_binary_inst!(self, not_equal),
                I32Or | I64Or => impl_binary_inst!(self, or),
                I32Xor | I64Xor => impl_binary_inst!(self, xor),
                I32And | I64And => impl_binary_inst!(self, and),
                If(_return_type, if_ops, else_ops) => {
                    let cond = &self.stack.pop_value();
                    if cond.is_truthy() {
                        let _ = self.evaluate_instructions(if_ops);
                    } else {
                        if !else_ops.is_empty() {
                            let _ = self.evaluate_instructions(else_ops);
                        }
                    }
                }
                Return => {
                    unimplemented!();
                }
                I64ExtendUnsignI32 => impl_unary_inst!(self, extend_to_i64),
                I32ShiftLeft | I64ShiftLeft => impl_binary_inst!(self, shift_left),
                I32ShiftRIghtSign | I64ShiftRightSign => impl_binary_inst!(self, shift_right_sign),
                I32ShiftRightUnsign | I64ShiftRightUnsign => {
                    impl_binary_inst!(self, shift_right_unsign)
                }
                I32WrapI64 => {
                    let i = &self.stack.pop_value();
                    match i {
                        Values::I64(n) => {
                            let result = (*n % 2_i64.pow(32)) as i32;
                            self.stack.push(StackEntry::new_value(Values::I32(result)));
                        }
                        x => unreachable!("Expected i64 value, got {:?}", x),
                    }
                }
                I32RotateLeft | I64RotateLeft => impl_binary_inst!(self, wasm_rotate_left),
                I32RotateRight | I64RotateRight => impl_binary_inst!(self, wasm_rotate_right),
                I32CountLeadingZero | I64CountLeadingZero => {
                    impl_unary_inst!(self, count_leading_zero)
                }
                I32CountTrailingZero | I64CountTrailingZero => {
                    impl_unary_inst!(self, count_trailing_zero)
                }
                I32CountNonZero | I64CountNonZero => impl_unary_inst!(self, pop_count),
                I32EqualZero | I64EqualZero => impl_unary_inst!(self, equal_zero),
                TypeEmpty => unreachable!(),

                I32Load8Unsign(_, offset) | I32Load8Sign(_, offset) => {
                    impl_load_inst!(8, self, offset, "i32")
                }
                I32Load16Unsign(_, offset) | I32Load16Sign(_, offset) => {
                    impl_load_inst!(16, self, offset, "i32")
                }
                I32Load(_, offset) => impl_load_inst!(32, self, offset, "i32"),
                I64Load8Unsign(_, offset) | I64Load8Sign(_, offset) => {
                    impl_load_inst!(8, self, offset, "i64")
                }
                I64Load16Unsign(_, offset) | I64Load16Sign(_, offset) => {
                    impl_load_inst!(16, self, offset, "i64")
                }
                I64Load32Sign(_, offset) | I64Load32Unsign(_, offset) => {
                    impl_load_inst!(32, self, offset, "i64")
                }
                I64Load(_, offset) => impl_load_inst!(64, self, offset, "i64"),
                F32Abs | F32Neg | F32Copysign => {
                    unimplemented!("{:?}", expression);
                }
                F32Load(_, offset) => impl_load_inst!(32, self, offset, "f32"),
                F64Load(_, offset) => impl_load_inst!(64, self, offset, "f64"),
                I32Store(_, _offset)
                | I64Store(_, _offset)
                | F32Store(_, _offset)
                | F64Store(_, _offset)
                | I32Store8(_, _offset)
                | I32Store16(_, _offset)
                | I64Store8(_, _offset)
                | I64Store16(_, _offset)
                | I64Store32(_, _offset) => {
                    unimplemented!("{:?}", expression);
                }
            };
        }
        Ok(())
    }

    fn evaluate_frame(&mut self, instructions: &Vec<Inst>) -> Result<()> {
        self.evaluate_instructions(instructions)?;
        let return_value = StackEntry::new_value(self.stack.pop_value());
        self.stack.update_frame_ptr();
        self.stack.push(return_value);
        Ok(())
    }

    fn call(&mut self, function_idx: usize, arguments: Vec<Values>) {
        let frame = StackEntry::new_fram(Frame {
            locals: arguments,
            return_ptr: self.stack.stack_ptr,
            function_idx,
        });
        self.stack.push(frame);
    }

    fn evaluate(&mut self) -> Result<()> {
        let mut result = None;
        while !self.stack.is_empty {
            let popped = self.stack.pop().expect("Invalid popping stack.");
            match *popped {
                StackEntry::Value(ref v) => {
                    result = Some(StackEntry::new_value(v.to_owned()));
                    break;
                }
                StackEntry::Label(ref expressions) => {
                    self.evaluate_frame(&expressions)?;
                }
                StackEntry::Frame(ref frame) => {
                    let _offset = frame.locals.len();
                    self.stack.frame_ptr.push(frame.return_ptr);
                    for local in frame.clone().locals {
                        self.stack.push(StackEntry::new_value(local));
                    }
                    let fn_instance = self.store.call(frame.function_idx);
                    let (expressions, locals) =
                        fn_instance.map(|f| f.call()).unwrap_or((vec![], vec![]));
                    let label = StackEntry::new_label(expressions);
                    self.stack.increase(locals.len());
                    self.stack.push(label);
                }
                StackEntry::Empty => unreachable!("Invalid popping stack."),
            }
        }
        self.stack
            .push(result.expect("Call stack may return with null value"));
        Ok(())
    }

    pub fn run(&mut self, invoke: &str, arguments: Vec<Values>) -> String {
        let start_idx = self.store.get_function_idx(invoke);
        self.call(start_idx, arguments);
        match self.evaluate() {
            Ok(_) => match self.stack.pop_value() {
                Values::I32(v) => format!("i32:{}", v),
                Values::I64(v) => format!("i64:{}", v),
                Values::F32(v) => format!("f32:{}", v),
                Values::F64(v) => format!("f64:{}", v),
            },
            Err(err) => String::from(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    macro_rules! test_eval {
        ($fn_name:ident, $file_name:expr, $call_arguments: expr, $expect_value: expr) => {
            #[test]
            fn $fn_name() {
                let mut file = File::open(format!("./dist/{}.wasm", $file_name)).unwrap();
                let mut buffer = vec![];
                let _ = file.read_to_end(&mut buffer);
                let mut vm = Vm::new(buffer).unwrap();
                let actual = vm.run("_subject", $call_arguments);
                assert_eq!(actual, format!("i32:{}", $expect_value));
            }
        };
    }

    #[test]
    fn repl() {
        println!("{}", std::f32::NAN);
        println!("{}", std::f32::INFINITY);
    }
    #[test]
    fn stack_ptr() {
        let mut stack = Stack::new(4);
        stack.push(StackEntry::new_value(Values::I32(1)));
        stack.set(2, StackEntry::new_value(Values::I32(2)));
        assert_eq!(*stack.pop().unwrap(), StackEntry::Value(Values::I32(1)));
        assert_eq!(*stack.get(2).unwrap(), StackEntry::Value(Values::I32(2)));
    }
    test_eval!(evaluate_cons8, "cons8", vec![], 42);
    test_eval!(
        evaluate_add_simple,
        "add",
        vec![Values::I32(3), Values::I32(4)],
        7
    );
    test_eval!(evaluate_sub, "sub", vec![Values::I32(10)], 90);
    test_eval!(
        evaluate_add_five,
        "add_five",
        vec![Values::I32(3), Values::I32(4)],
        17
    );
    test_eval!(evaluate_if_lt_1, "if_lt", vec![Values::I32(10)], 15);
    test_eval!(evaluate_if_lt_2, "if_lt", vec![Values::I32(9)], 19);
    test_eval!(evaluate_if_lt_3, "if_lt", vec![Values::I32(11)], 26);

    test_eval!(evaluate_if_gt_1, "if_gt", vec![Values::I32(10)], 15);
    test_eval!(evaluate_if_gt_2, "if_gt", vec![Values::I32(15)], 25);
    test_eval!(evaluate_if_gt_3, "if_gt", vec![Values::I32(5)], 20);

    test_eval!(evaluate_if_eq_1, "if_eq", vec![Values::I32(10)], 15);
    test_eval!(evaluate_if_eq_2, "if_eq", vec![Values::I32(11)], 21);
    test_eval!(evaluate_fib, "fib", vec![Values::I32(15)], 610);
    test_eval!(evaluate_5_count, "count", vec![Values::I32(5)], 35);
    test_eval!(evaluate_10_count, "count", vec![Values::I32(10)], 145);
    test_eval!(evaluate_100_count, "count", vec![Values::I32(100)], 14950);
}
