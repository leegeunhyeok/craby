// Implement module for 'Basic' here
use crate::generated::BasicSpec;

pub struct Basic;

impl BasicSpec for Basic {
    fn numeric_method(arg: f64) -> f64 {
        arg * 2.0
    }

    fn boolean_method(arg: bool) -> bool {
        !arg
    }
}
