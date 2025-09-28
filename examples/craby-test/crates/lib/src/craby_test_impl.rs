use crate::ffi;
use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct CrabyTest {
    id: usize,
}

impl CrabyTestSpec for CrabyTest {
    fn new(id: usize) -> CrabyTest {
        CrabyTest { id }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn numeric_method(&self, arg: Number) -> Number {
        arg * 2.0
    }

    fn boolean_method(&self, arg: Boolean) -> Boolean {
        !arg
    }

    fn string_method(&self, arg: String) -> String {
        format!("From Rust: {}", arg)
    }

    fn object_method(&self, mut arg: TestObject) -> TestObject {
        arg.foo = format!("From Rust: {}", arg.foo);
        arg.bar = arg.bar * 2.0;
        arg.baz = !arg.baz;
        arg
    }

    fn array_method(&self, mut arg: Array<Number>) -> Array<Number> {
        arg.extend(vec![1.0, 2.0, 3.0]);
        arg.iter_mut().for_each(|x| *x *= 2.0);
        arg
    }

    fn enum_method(&self, arg0: MyEnum, arg1: SwitchState) -> String {
        let arg0 = match arg0 {
            MyEnum::Foo => "Enum Foo!",
            MyEnum::Bar => "Enum Bar!",
            MyEnum::Baz => "Enum Baz!",
            _ => unreachable!(),
        };

        let arg1 = match arg1 {
            SwitchState::Off => "Off",
            SwitchState::On => "On",
            _ => unreachable!(),
        };

        format!("Enum {} / {}", arg0, arg1)
    }

    fn nullable_method(&self, arg: Nullable<Number>) -> Nullable<Number> {
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

    fn promise_method(&self, arg: Number) -> Promise<Number> {
        if arg >= 0.0 {
            promise::resolve(arg * 2.0)
        } else {
            promise::rejected("Boom!")
        }
    }

    fn trigger_signal(&self) -> Void {
        self.emit(CrabyTestSignal::OnSignal);
    }
}
