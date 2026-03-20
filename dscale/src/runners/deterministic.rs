use std::sync::Arc;

use crate::{
    ProcessHandle,
    actor::Actors,
    global::{
        self,
        configuration::setup_local_configuration,
        local_access::{self, setup_local_access},
    },
    global_unique_id,
    random::Seed,
    runners::{SimulationRunner, emojis::deadlock, progress::Bar, task::TaskResult},
    step::Step,
    time::Jiffies,
};

pub struct DeterministicRunner {
    actors: Actors,
    time_budget: Jiffies,
    procs: Vec<Arc<dyn ProcessHandle>>,
    progress_bar: Bar,
}

impl DeterministicRunner {
    pub(crate) fn new(
        actors: Actors,
        time_budget: Jiffies,
        procs: Vec<Arc<dyn ProcessHandle>>,
        seed: Seed,
    ) -> Self {
        for id in 0..procs.len() {
            setup_local_configuration(id, seed);
        }
        // Set up thread-local access on the main thread directly — no channel needed.
        // We pass a dummy sender that is never used since we call take_events() instead of done().
        let (tx, _rx) = crossbeam_channel::unbounded::<TaskResult>();
        setup_local_access(seed, tx);
        Self {
            actors,
            time_budget,
            progress_bar: Bar::new(time_budget),
            procs,
        }
    }
}

impl SimulationRunner for DeterministicRunner {
    fn run_full_budget(&mut self) {
        self.start();

        while global::now() < self.time_budget {
            self.run_next_step();
        }

        self.progress_bar.finish();
    }
}

impl DeterministicRunner {
    fn start(&mut self) {
        for proc_id in 0..self.procs.len() {
            self.run_step(Step::Start { to: proc_id });
        }
    }

    fn run_next_step(&mut self) {
        if let Some(next_time) = self.actors.peek_next_step() {
            global::fast_forward_clock(next_time);
            self.progress_bar.make_progress(next_time);
            let step = self.actors.next_step();
            self.run_step(step);
        } else {
            deadlock();
        }
    }

    fn run_step(&mut self, step: Step) {
        let task_id = (global::now(), global_unique_id());
        match step {
            Step::Start { to } => {
                local_access::set_task(task_id, to);
                self.procs[to].start();
            }
            Step::NetworkStep { from, to, message } => {
                local_access::set_task(task_id, to);
                self.procs[to].on_message(from, message);
            }
            Step::TimerStep { to, id } => {
                local_access::set_task(task_id, to);
                self.procs[to].on_timer(id);
            }
        }
        let mut events = local_access::take_events();
        self.actors.submit(&mut events);
    }
}
