use crate::{ffi::ffi::*, generated::*};

pub struct CrabyTest;

impl CrabyTestSpec for CrabyTest {
    fn numeric_method(arg: f64) -> f64 {
        arg * 2.0
    }

    fn boolean_method(arg: bool) -> bool {
        !arg
    }

    fn string_method(arg: String) -> String {
        format!("From Rust: {}", arg)
    }

    fn object_method(arg: TestObject) -> TestObject {
        TestObject {
          foo: format!("From Rust: {}", arg.foo),
          bar: arg.bar * 2.0,
          baz: !arg.baz,
        }
    }
}
