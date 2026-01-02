use std::{cell::RefCell, rc::Rc};

use crate::time::Jiffies;

pub(crate) type SharedActor = Rc<RefCell<dyn SimulationActor>>;

pub(crate) trait SimulationActor {
    fn Start(&mut self);
    fn Step(&mut self);
    fn PeekClosest(&self) -> Option<Jiffies>;
}
