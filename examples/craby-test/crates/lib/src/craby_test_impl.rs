use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct CrabyTest;

impl CrabyTestSpec for CrabyTest {
    fn numeric_method(arg: Number) -> Number {
        arg * 2.0
    }

    fn boolean_method(arg: Boolean) -> Boolean {
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

    fn array_method(mut arg: Array<Number>) -> Array<Number> {
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

    fn nullable_method(arg: Nullable<Number>) -> Nullable<Number> {
        match arg.value_of() {
            Some(val) => {
                if *val < 0.0 {
                    Nullable::<Number>::none()
                } else {
                    let new_val = val * 10.0;
                    arg.value(new_val)
                }
            }
            None => Nullable::<Number>::some(123.0),
        }
    }

    fn promise_method(arg: Number) -> Promise<Number> {
        if arg >= 0.0 {
            promise::resolve(arg * 2.0)
        } else {
            promise::rejected("Boom!")
        }
    }
}
