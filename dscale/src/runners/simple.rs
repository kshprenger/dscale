use crate::{
    ProcessHandle,
    actors::Actors,
    global::{
        self,
        configuration::setup_local_configuration,
        local_access::{self, setup_local_access},
    },
    global_unique_id,
    jiffy::Jiffies,
    random::Seed,
    runners::{RunStatus, SimulationRunner, progress::Bar, task::TaskResult},
    step::Step,
};

pub(crate) struct SimpleRunner {
    actors: Actors,
    time_budget: Jiffies,
    procs: Vec<Box<dyn ProcessHandle>>,
    progress_bar: Bar,
    started: bool,
}

impl SimpleRunner {
    pub(crate) fn new(
        actors: Actors,
        time_budget: Jiffies,
        procs: Vec<Box<dyn ProcessHandle>>,
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
            started: false,
        }
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.started = true;
            for rank in 0..self.procs.len() {
                self.run_step(Step::Start { rank });
            }
        }
    }
}

impl Drop for SimpleRunner {
    fn drop(&mut self) {
        global::reset();
    }
}

impl SimulationRunner for SimpleRunner {
    fn run_full_budget(&mut self) -> RunStatus {
        self.ensure_started();

        let mut steps = 0;
        while global::now() < self.time_budget {
            if self.actors.peek_next_step().is_none() {
                return RunStatus::NoMoreEvents { steps };
            }
            self.run_next_step();
            steps += 1;
        }

        self.progress_bar.finish();
        RunStatus::BudgetExhausted { steps }
    }

    fn run_steps(&mut self, k: usize) -> RunStatus {
        self.ensure_started();

        let mut steps = 0;
        while steps < k {
            if global::now() >= self.time_budget {
                return RunStatus::BudgetExhausted { steps };
            }
            if self.actors.peek_next_step().is_none() {
                return RunStatus::NoMoreEvents { steps };
            }
            self.run_next_step();
            steps += 1;
        }
        RunStatus::Completed { steps }
    }

    fn run_sub_budget(&mut self, sub_budget: Jiffies) -> RunStatus {
        self.ensure_started();

        let deadline = global::now() + sub_budget;
        let mut steps = 0;
        while global::now() < deadline {
            if global::now() >= self.time_budget {
                return RunStatus::BudgetExhausted { steps };
            }
            if self.actors.peek_next_step().is_none() {
                return RunStatus::NoMoreEvents { steps };
            }
            self.run_next_step();
            steps += 1;
        }
        RunStatus::Completed { steps }
    }
}

impl SimpleRunner {
    fn run_next_step(&mut self) {
        let next_time = self.actors.peek_next_step().expect("checked by caller");
        global::fast_forward_clock(next_time);
        self.progress_bar.make_progress(next_time);
        let step = self.actors.next_step();
        self.run_step(step);
    }

    fn run_step(&mut self, step: Step) {
        let task_id = (global::now(), global_unique_id());
        match step {
            Step::Start { rank } => {
                local_access::set_task(task_id, rank);
                self.procs[rank].on_start();
            }
            Step::NetworkStep {
                source,
                target,
                message,
            } => {
                local_access::set_task(task_id, target);
                self.procs[target].on_message(source, message);
            }
            Step::TimerStep { rank, id } => {
                local_access::set_task(task_id, rank);
                self.procs[rank].on_timer(id);
            }
        }
        let mut events = local_access::take_events();
        self.actors.submit(&mut events);
    }
}
