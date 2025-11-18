use crate::simulation_handle::SIMULATION_HANDLE;

pub type Jiffies = usize;

pub fn schedule_timeout(after: Jiffies) {
    SIMULATION_HANDLE.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .map(|sim| sim.submit_event(crate::communication::Event::Timeout(after)))
            .or_else(|| panic!("Out of simulation context"))
    });
}
