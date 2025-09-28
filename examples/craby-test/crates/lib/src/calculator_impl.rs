use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct Calculator {
    id: usize,
}

impl CalculatorSpec for Calculator {
    fn new(id: usize) -> Calculator {
        Calculator { id }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn add(&self, a: Number, b: Number) -> Number {
        a + b
    }

    fn subtract(&self, a: Number, b: Number) -> Number {
        a - b
    }

    fn multiply(&self, a: Number, b: Number) -> Number {
        a * b
    }

    fn divide(&self, a: Number, b: Number) -> Number {
        a / b
    }
}
