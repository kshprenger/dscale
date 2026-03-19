use std::{cmp::Reverse, process::exit, sync::Arc, time::Duration, usize};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use log::{error, info};

use crate::{
    ProcessHandle,
    actor::SimulationActor,
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
    actors: Vec<Box<dyn SimulationActor>>,
    rx: Receiver<TaskResult>,
    time_budget: Jiffies,
    procs: Vec<Arc<dyn ProcessHandle>>,
    workers: rayon::ThreadPool,
    window_l: Jiffies,
    window_r: Jiffies,
    on_execution: TaskIndex,
    done: TaskIndex,
}

impl ScalableRunner {
    pub(crate) fn new(
        actors: Vec<Box<dyn SimulationActor>>,
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
            window_r: safe_window,
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
                self.workers.install(move || {
                    local_access::set_task(task_id, proc_id);
                    proc.start();
                    local_access::ready();
                });
            });
    }

    fn coordinate(&mut self) {
        loop {
            match self.rx.recv_timeout(DEADLOCK_TIMEOUT) {
                Ok(task_result) => {
                    if global::now() > self.time_budget {
                        return;
                    }
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

    fn peek_closest(&mut self) -> Option<(Jiffies, SharedActor)> {
        let mut min_time = Jiffies(usize::MAX);
        let mut sha: Option<SharedActor> = None;
        for actor in self.actors.iter() {
            actor
                .lock()
                .expect("Actor lock poisoned")
                .peek_closest()
                .map(|time| {
                    if time < min_time {
                        min_time = time;
                        sha = Some(actor.clone())
                    }
                });
        }

        Some((min_time, sha?))
    }
}
