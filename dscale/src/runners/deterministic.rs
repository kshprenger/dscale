use crate::{
    actor::Actors,
    global,
    runners::{SimulationRunner, emojis::deadlock, progress::Bar, workers::Workers},
    step::Step,
    time::Jiffies,
};

pub struct DeterministicRunner {
    actors: Actors,
    time_budget: Jiffies,
    progress_bar: Bar,
    workers: Workers,
}

impl DeterministicRunner {
    pub(crate) fn new(actors: Actors, time_budget: Jiffies, workers: Workers) -> Self {
        Self {
            actors,
            time_budget,
            progress_bar: Bar::new(time_budget),
            workers,
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
        for proc_id in 0..self.workers.num_procs() {
            self.workers.spawn_step(Step::Start { to: proc_id });
            let mut task_result = self.workers.recv();
            self.actors.submit(&mut task_result.events);
        }
    }

    fn run_next_step(&mut self) {
        if let Some(next_time) = self.actors.peek_next_step() {
            global::fast_forward_clock(next_time);
            self.progress_bar.make_progress(next_time);
            let step = self.actors.next_step();
            self.workers.spawn_step(step);
            let mut task_result = self.workers.recv();
            self.actors.submit(&mut task_result.events);
        } else {
            deadlock();
        }
    }
}
