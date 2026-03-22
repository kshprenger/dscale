use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crossbeam_channel::{Receiver, RecvTimeoutError};

use crate::{
    ProcessHandle,
    global::{
        self,
        configuration::setup_local_configuration,
        local_access::{self, setup_local_access},
    },
    global_unique_id,
    random::Seed,
    runners::task::{TaskId, TaskResult},
    step::Step,
};

pub(crate) struct Workers {
    procs: Vec<Arc<Mutex<dyn ProcessHandle + Send>>>,
    pool: rayon::ThreadPool,
    rx: Receiver<TaskResult>,
}

impl Workers {
    pub(crate) fn new(
        procs: Vec<Arc<Mutex<dyn ProcessHandle + Send>>>,
        cores: usize,
        seed: Seed,
    ) -> Self {
        for id in 0..procs.len() {
            setup_local_configuration(id, seed);
        }
        let (tx, rx) = crossbeam_channel::unbounded::<TaskResult>();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(cores)
            .start_handler(move |_| {
                setup_local_access(seed, tx.clone());
            })
            .build()
            .expect("Could not build thread pool");
        Self { procs, pool, rx }
    }

    pub(crate) fn num_procs(&self) -> usize {
        self.procs.len()
    }

    pub(crate) fn spawn_step(&self, step: Step) -> TaskId {
        let task_id = (global::now(), global_unique_id());
        match step {
            Step::Start { rank } => {
                self.spawn_on_worker(task_id, rank, move |proc| proc.on_start());
            }
            Step::NetworkStep {
                source,
                target,
                message,
            } => {
                self.spawn_on_worker(task_id, target, move |proc| {
                    proc.on_message(source, message)
                });
            }
            Step::TimerStep { rank, id } => {
                self.spawn_on_worker(task_id, rank, move |proc| proc.on_timer(id));
            }
        }
        task_id
    }

    pub(crate) fn try_recv(&self) -> Option<TaskResult> {
        self.rx.try_recv().ok()
    }

    pub(crate) fn recv_timeout(&self, timeout: Duration) -> Result<TaskResult, RecvTimeoutError> {
        self.rx.recv_timeout(timeout)
    }

    fn spawn_on_worker(
        &self,
        task_id: TaskId,
        proc_id: usize,
        work: impl FnOnce(&mut dyn ProcessHandle) + Send + 'static,
    ) {
        let proc = self.procs[proc_id].clone();
        self.pool.spawn(move || {
            local_access::set_task(task_id, proc_id);
            let mut guard = proc.lock().unwrap();
            work(&mut *guard);
            drop(guard);
            local_access::done();
        });
    }
}
