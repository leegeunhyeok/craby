use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct {{ module_name }} {
    id: usize,
}

impl {{ module_name }}Spec for {{ module_name }} {
    fn new(id: usize) -> {{ module_name }} {
        {{ module_name }} { id }
    }

    fn id(&self) -> usize {
        self.id
    }
}
