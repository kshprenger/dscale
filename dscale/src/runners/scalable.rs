use std::{cmp::Reverse, process::exit, sync::Arc, time::Duration, usize};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use log::{error, info};

use crate::{
    ProcessHandle,
    actor::Actors,
    global::{
        self,
        local_access::{self, setup_local_access},
    },
    global_unique_id,
    random::Seed,
    runner::SimulationRunner,
    runners::{
        emojis,
        task::{TaskIndex, TaskResult},
    },
    time::Jiffies,
};

const DEADLOCK_TIMEOUT: Duration = Duration::from_millis(2000);

pub struct ScalableRunner {
    actors: Actors,
    rx: Receiver<TaskResult>,
    time_budget: Jiffies,
    procs: Vec<Arc<dyn ProcessHandle>>,
    workers: rayon::ThreadPool,
    window_l: Jiffies,
    window_delta: Jiffies,
    on_execution: TaskIndex,
    done: TaskIndex,
}

impl ScalableRunner {
    pub(crate) fn new(
        actors: Actors,
        time_budget: Jiffies,
        procs: Vec<Arc<dyn ProcessHandle>>,
        cores: usize,
        seed: Seed,
        safe_window: Jiffies,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<TaskResult>();
        let tp = rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .start_handler(move |_| {
                setup_local_access(seed, tx.clone());
            })
            .build()
            .expect("Could not build tp");
        Self {
            actors,
            time_budget,
            procs,
            rx,
            workers: tp,
            window_l: Jiffies(0),
            window_delta: safe_window,
            on_execution: TaskIndex::new(),
            done: TaskIndex::new(),
        }
    }
}

impl SimulationRunner for ScalableRunner {
    fn run_full_budget(&mut self) {
        self.start();
        self.coordinate();
        info!("Looks good! {}", emojis::good());
    }
}

impl ScalableRunner {
    fn start(&mut self) {
        let task_id = (Jiffies(0), global_unique_id());
        self.on_execution.push(Reverse(task_id));
        self.procs
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(proc_id, proc)| {
                self.workers.spawn(move || {
                    local_access::set_task(task_id, proc_id);
                    proc.start();
                    local_access::ready();
                });
            });
    }

    fn coordinate(&mut self) {
        loop {
            match self.rx.recv_timeout(DEADLOCK_TIMEOUT) {
                Ok(mut task_result) => {
                    if global::now() > self.time_budget {
                        return;
                    }
                    self.actors.submit(&mut task_result.events); // Materialize next part of dependency graph
                    self.done.push(Reverse(task_result.id));
                    self.try_extract();
                    self.try_advance()
                }
                Err(RecvTimeoutError::Timeout) => {
                    error!(
                        "DEADLOCK! {}\nTry using deterministic runner with RUST_LOG=debug",
                        emojis::bad()
                    );
                    exit(1)
                }
                Err(RecvTimeoutError::Disconnected) => {
                    unreachable!("ooops")
                }
            }
        }
    }

    fn try_extract(&mut self) {
        while let (Some(d), Some(e)) = (self.done.peek(), self.on_execution.peek()) {
            if d == e {
                self.done.pop();
                self.on_execution.pop();
            } else {
                break;
            }
        }
    }

    fn try_advance(&mut self) {
        self.window_l = self.actors.peek_next_step().unwrap();

        while let Some(step) = self.actors.peek_next_step() {
            // if step
        }
    }
}
