use craby::{prelude::*, throw};

use crate::ffi::bridging::*;
use crate::generated::*;

pub struct Calculator {
    ctx: Context,
}

#[craby_module]
impl CalculatorSpec for Calculator {
    fn add(&mut self, a: Number, b: Number) -> Number {
        a + b
    }

    fn subtract(&mut self, a: Number, b: Number) -> Number {
        a - b
    }

    fn multiply(&mut self, a: Number, b: Number) -> Number {
        a * b
    }

    fn divide(&mut self, a: Number, b: Number) -> Number {
        if b == 0.0 {
            throw!("Division by zero");
        }
        a / b
    }
}
