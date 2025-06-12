use marwood::vm::Vm;

pub struct Marwood {
    pub vm: Vm,
}

impl Marwood {
    pub fn new() -> Self {
        Marwood { vm: Vm::new() }
    }
}
