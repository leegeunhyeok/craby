use crate::{ffi::craby_test::*, generated::*};

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

    fn object_method(mut arg: TestObject) -> TestObject {
        arg.foo = format!("From Rust: {}", arg.foo);
        arg.bar = arg.bar * 2.0;
        arg.baz = !arg.baz;
        arg
    }

    fn array_method(mut arg: Vec<f64>) -> Vec<f64> {
        arg.extend(vec![1.0, 2.0, 3.0]);
        arg.iter_mut().for_each(|x| *x *= 2.0);
        arg
    }

    fn enum_method(arg: MyEnum) -> String {
        match arg {
            MyEnum::FOO => "FOO!".to_string(),
            MyEnum::BAR => "BAR!".to_string(),
            MyEnum::BAZ => "BAZ!".to_string(),
            _ => unreachable!(),
        }
    }

    fn promise_method(arg: f64) -> Result<f64, anyhow::Error> {
        if arg >= 0.0 {
          Ok(arg * 2.0)
        } else {
          Err(anyhow::anyhow!("Boom!"))
        }
    }
}
