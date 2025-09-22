use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct Calculator;

impl CalculatorSpec for Calculator {
    fn add(a: Number, b: Number) -> Number {
        a + b
    }

    fn subtract(a: Number, b: Number) -> Number {
        a - b
    }

    fn multiply(a: Number, b: Number) -> Number {
        a * b
    }

    fn divide(a: Number, b: Number) -> Number {
        a / b
    }
}
