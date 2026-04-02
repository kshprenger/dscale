use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, RecvError};

use crate::{
    ProcessHandle,
    global::{
        configuration::setup_local_configuration,
        local_access::{self, setup_local_access},
    },
    random::Seed,
    runners::{
        threads::Threads,
        task::{TaskId, TaskResult},
    },
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
        threads: Threads,
        seed: Seed,
    ) -> Self {
        for id in 0..procs.len() {
            setup_local_configuration(id, seed);
        }
        let threads_number: usize = threads.into();
        let (tx, rx) = crossbeam_channel::unbounded::<TaskResult>();
        log::warn!("Using {threads_number} threads for simulation");
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads_number)
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

    pub(crate) fn spawn_step(&self, task_id: TaskId, step: Step) {
        let (proc_id, work) = Self::step_into_work(step);
        let proc = self.procs[proc_id].clone();
        self.pool.spawn(Self::wrap(task_id, proc_id, proc, work));
    }

    pub(crate) fn install_step(&self, task_id: TaskId, step: Step) {
        let (proc_id, work) = Self::step_into_work(step);
        let proc = self.procs[proc_id].clone();
        self.pool.install(Self::wrap(task_id, proc_id, proc, work));
    }

    pub(crate) fn try_next_result(&self) -> Option<TaskResult> {
        self.rx.try_recv().ok()
    }

    pub(crate) fn next_result(&self) -> Result<TaskResult, RecvError> {
        self.rx.recv()
    }

    fn step_into_work(step: Step) -> (usize, Box<dyn FnOnce(&mut dyn ProcessHandle) + Send>) {
        match step {
            Step::Start { rank } => (rank, Box::new(|proc| proc.on_start())),
            Step::NetworkStep {
                source,
                target,
                message,
            } => (
                target,
                Box::new(move |proc| proc.on_message(source, message)),
            ),
            Step::TimerStep { rank, id } => (rank, Box::new(move |proc| proc.on_timer(id))),
        }
    }

    fn wrap(
        task_id: TaskId,
        proc_id: usize,
        proc: Arc<Mutex<dyn ProcessHandle + Send>>,
        work: Box<dyn FnOnce(&mut dyn ProcessHandle) + Send>,
    ) -> impl FnOnce() + Send {
        move || {
            local_access::set_task(task_id, proc_id);
            let mut guard = proc.lock().unwrap();
            work(&mut *guard);
            drop(guard);
            local_access::done();
        }
    }
}
