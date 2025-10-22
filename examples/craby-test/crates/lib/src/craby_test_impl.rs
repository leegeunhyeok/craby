use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct CrabyTest {
    id: usize,
    state: Option<Number>,
}

impl CrabyTestSpec for CrabyTest {
    fn new(id: usize) -> Self {
        CrabyTest { id, state: None }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn numeric_method(&mut self, arg: Number) -> Number {
        arg * 2.0
    }

    fn boolean_method(&mut self, arg: Boolean) -> Boolean {
        !arg
    }

    fn string_method(&mut self, arg: &str) -> String {
        format!("From Rust: {}", arg.to_string())
    }

    fn object_method(&mut self, mut arg: TestObject) -> TestObject {
        arg.foo = format!("From Rust: {}", arg.foo);
        arg.bar = arg.bar * 2.0;
        arg.baz = !arg.baz;
        arg.camel_case = arg.camel_case + 1.0;
        arg.pascal_case = arg.pascal_case + 1.0;
        arg.snake_case = arg.snake_case + 1.0;
        arg
    }

    fn array_method(&mut self, mut arg: Array<Number>) -> Array<Number> {
        arg.extend(vec![1.0, 2.0, 3.0]);
        arg.iter_mut().for_each(|x| *x *= 2.0);
        arg
    }

    fn enum_method(&mut self, arg0: MyEnum, arg1: SwitchState) -> String {
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

    fn nullable_method(&mut self, arg: Nullable<Number>) -> Nullable<Number> {
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

    fn promise_method(&mut self, arg: Number) -> Promise<Number> {
        if arg == 0.0 {
            throw!("Zero is not allowed");
        }

        if arg >= 0.0 {
            promise::resolve(arg * 2.0)
        } else {
            promise::reject("Boom!")
        }
    }

    fn set_state(&mut self, arg: Number) -> Void {
        self.state = Some(arg);
    }

    fn get_state(&mut self) -> Number {
        self.state.unwrap_or(0.0)
    }

    fn trigger_signal(&mut self) -> Void {
        self.emit(CrabyTestSignal::OnSignal);
    }

    fn camel_method(&mut self) -> Void {
        // noop
    }

    fn pascal_method(&mut self) -> Void {
        // noop
    }

    fn snake_method(&mut self) -> Void {
        // noop
    }
}
