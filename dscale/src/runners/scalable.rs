use std::{cmp::Reverse, sync::Arc, time::Duration, usize};

use crossbeam_channel::{Receiver, RecvTimeoutError};

use crate::{
    ProcessHandle,
    actor::Actors,
    global::{
        self,
        local_access::{self, setup_local_access},
    },
    global_unique_id, now,
    random::Seed,
    runners::{
        SimulationRunner,
        emojis::{deadlock, looks_good},
        task::{TaskId, TaskIndex, TaskResult},
    },
    step::Step,
    time::Jiffies,
};

const DEADLOCK_TIMEOUT: Duration = Duration::from_millis(2000);

pub struct ScalableRunner {
    actors: Actors,
    rx: Receiver<TaskResult>,
    time_budget: Jiffies,
    procs: Vec<Arc<dyn ProcessHandle>>,
    workers: rayon::ThreadPool,
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
        looks_good();
    }
}

impl ScalableRunner {
    fn start(&mut self) {
        for proc_id in 0..self.procs.len() {
            self.spawn_step_with(Jiffies(0), Step::Start { to: proc_id });
        }
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
                    self.adjust_task_index();
                    self.try_advance()
                }
                Err(RecvTimeoutError::Timeout) => deadlock(),
                Err(RecvTimeoutError::Disconnected) => {
                    unreachable!("ooops")
                }
            }
        }
    }

    fn adjust_task_index(&mut self) {
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
        if self.try_move_window() {
            self.spawn_remain_within_window()
        }
    }

    fn try_move_window(&mut self) -> bool {
        if let Some(top) = self.on_execution.peek() {
            // Still waiting some tasks on execution in current window
            if global::now() == top.0.0 {
                // Lowest task(s) still on execution
                // Cannot move window forward, need to wait lowest task(s)
                return false;
            } else {
                // Lowest task(s) in window done -> advance window to the next closest task (which is still on_execution)
                global::fast_forward_clock(top.0.0);
            }
        } else {
            // done == on_execution
            // In this case we should move window to the next materialized tasks
            // If no such materialized task exists -> deadlock
            if let Some(next_step_invocation_time) = self.actors.peek_next_step() {
                global::fast_forward_clock(next_step_invocation_time);
            } else {
                deadlock();
            }
        }

        return true;
    }

    fn spawn_remain_within_window(&mut self) {
        while let Some(next_step_invocation_time) = self.actors.peek_next_step() {
            if next_step_invocation_time - now() <= self.window_delta {
                let next_step = self.actors.next_step();
                self.spawn_step_with(next_step_invocation_time, next_step);
            }
        }
    }

    fn spawn_step_with(&mut self, step_invocation_time: Jiffies, step: Step) {
        let task_id = (step_invocation_time, global_unique_id());
        self.on_execution.push(Reverse(task_id));
        match step {
            Step::Start { to } => {
                self.spawn_on_worker(task_id, to, move |proc| proc.start());
            }
            Step::NetworkStep { from, to, message } => {
                self.spawn_on_worker(task_id, to, move |proc| proc.on_message(from, message));
            }
            Step::TimerStep { to, id } => {
                self.spawn_on_worker(task_id, to, move |proc| proc.on_timer(id));
            }
        }
    }

    fn spawn_on_worker(
        &self,
        task_id: TaskId,
        proc_id: usize,
        work: impl FnOnce(Arc<dyn ProcessHandle>) + Send + 'static,
    ) {
        let proc = self.procs[proc_id].clone();
        self.workers.spawn(move || {
            local_access::set_task(task_id, proc_id);
            work(proc);
            local_access::done();
        });
    }
}
