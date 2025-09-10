use crate::generated::CrabyTestSpec;

pub struct CrabyTest;

impl CrabyTestSpec for CrabyTest {
    fn numeric_method(arg: f64) -> f64 {
        arg * 2.0
    }

    fn boolean_method(arg: bool) -> bool {
        !arg
    }
}
