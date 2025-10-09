use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct Calculator {
    id: usize,
}

impl CalculatorSpec for Calculator {
    fn new(id: usize) -> Self {
        Calculator { id }
    }

    fn id(&self) -> usize {
        self.id
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
