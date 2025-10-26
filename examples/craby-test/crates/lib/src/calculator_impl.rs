use crate::ffi::bridging::*;
use crate::generated::*;
use crate::context::*;
use crate::types::*;

pub struct Calculator {
    ctx: Context,
}

impl CalculatorSpec for Calculator {
    fn new(ctx: Context) -> Self {
        Calculator { ctx }
    }

    fn id(&self) -> usize {
        self.ctx.id
    }

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
