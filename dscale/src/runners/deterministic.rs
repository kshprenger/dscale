use std::sync::Arc;

use crossbeam_channel::Receiver;

use crate::{
    ProcessHandle,
    actor::Actors,
    global::{
        self,
        local_access::{self, setup_local_access},
    },
    global_unique_id,
    random::Seed,
    runners::{
        SimulationRunner,
        progress::Bar,
        task::{TaskId, TaskResult},
    },
    step::Step,
    time::Jiffies,
};

pub struct DeterministicRunner {
    actors: Actors,
    time_budget: Jiffies,
    procs: Vec<Arc<dyn ProcessHandle>>,
    progress_bar: Bar,
    workers: rayon::ThreadPool,
    rx: Receiver<TaskResult>,
}

impl DeterministicRunner {
    pub(crate) fn new(
        actors: Actors,
        time_budget: Jiffies,
        procs: Vec<Arc<dyn ProcessHandle>>,
        seed: Seed,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<TaskResult>();
        let tp = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .start_handler(move |_| {
                setup_local_access(seed, tx.clone());
            })
            .build()
            .expect("Could not build tp");
        Self {
            actors,
            time_budget,
            progress_bar: Bar::new(time_budget),
            procs,
            rx,
            workers: tp,
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
            let mut task_result = self.rx.recv().expect("Worker disconnected");
            self.actors.submit(&mut task_result.events);
        }
    }

    fn run_next_step(&mut self) {
        if let Some(next_time) = self.actors.peek_next_step() {
            global::fast_forward_clock(next_time);
            self.progress_bar.make_progress(next_time);
            let step = self.actors.next_step();
            self.run_step(step);
            let mut task_result = self.rx.recv().expect("Worker disconnected");
            self.actors.submit(&mut task_result.events);
        }
    }

    fn run_step(&self, step: Step) {
        match step {
            Step::Start { to } => {
                self.spawn_on_worker(to, move |proc| proc.start());
            }
            Step::NetworkStep { from, to, message } => {
                self.spawn_on_worker(to, move |proc| proc.on_message(from, message));
            }
            Step::TimerStep { to, id } => {
                self.spawn_on_worker(to, move |proc| proc.on_timer(id));
            }
        }
    }

    fn spawn_on_worker(
        &self,
        proc_id: usize,
        work: impl FnOnce(Arc<dyn ProcessHandle>) + Send + 'static,
    ) {
        let task_id: TaskId = (global::now(), global_unique_id());
        let proc = self.procs[proc_id].clone();
        self.workers.spawn(move || {
            local_access::set_task(task_id, proc_id);
            work(proc);
            local_access::done();
        });
    }
}
