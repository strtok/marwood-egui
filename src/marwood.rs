use std::cell::RefCell;
use std::rc::Rc;

use marwood::cell::Cell;
use marwood::vm::{SystemInterface, Vm};

#[derive(Debug, Default)]
pub struct EguiSystem {
    buffer: Rc<RefCell<String>>,
}

impl EguiSystem {
    pub fn buffer(&self) -> Rc<RefCell<String>> {
        self.buffer.clone()
    }
}

impl SystemInterface for EguiSystem {
    fn display(&self, cell: &Cell) {
        self.buffer.borrow_mut().push_str(&format!("{}", cell));
    }

    fn write(&self, cell: &Cell) {
        self.buffer.borrow_mut().push_str(&format!("{:?}", cell));
    }

    fn terminal_dimensions(&self) -> (usize, usize) {
        (80, 24)
    }

    fn time_utc(&self) -> u64 {
        0
    }
}

pub struct Marwood {
    pub vm: Vm,
    pub display_buffer: Rc<RefCell<String>>,
}

impl Default for Marwood {
    fn default() -> Self {
        Self::new()
    }
}

impl Marwood {
    pub fn new() -> Self {
        let sys = EguiSystem::default();
        let display_buffer = sys.buffer();
        let mut vm = Vm::new();
        vm.set_system_interface(Box::new(sys));
        Marwood { vm, display_buffer }
    }
}
