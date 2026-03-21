use std::{cmp::Reverse, time::Duration};

use crossbeam_channel::RecvTimeoutError;

use crate::{
    actor::Actors,
    global::{self},
    now,
    runners::{
        SimulationRunner,
        emojis::{deadlock, looks_good},
        progress::Bar,
        task::{TaskIndex, TaskResult},
        workers::Workers,
    },
    step::Step,
    time::Jiffies,
};

const DEADLOCK_TIMEOUT: Duration = Duration::from_millis(2000);

pub struct ScalableRunner {
    actors: Actors,
    time_budget: Jiffies,
    workers: Workers,
    progress_bar: Bar,
    window_delta: Jiffies,
    on_execution: TaskIndex,
    done: TaskIndex,
}

impl ScalableRunner {
    pub(crate) fn new(
        actors: Actors,
        time_budget: Jiffies,
        workers: Workers,
        safe_window: Jiffies,
    ) -> Self {
        Self {
            actors,
            time_budget,
            workers,
            progress_bar: Bar::new(time_budget),
            window_delta: safe_window,
            on_execution: TaskIndex::new(),
            done: TaskIndex::new(),
        }
    }
}

impl Drop for ScalableRunner {
    fn drop(&mut self) {
        global::reset();
    }
}

impl SimulationRunner for ScalableRunner {
    fn run_full_budget(&mut self) {
        self.start();
        self.coordinate();
        self.progress_bar.finish();
        looks_good();
    }
}

impl ScalableRunner {
    fn start(&mut self) {
        for proc_id in 0..self.workers.num_procs() {
            let task_id = self.workers.spawn_step(Step::Start { to: proc_id });
            self.on_execution.push(Reverse(task_id));
        }
    }

    fn coordinate(&mut self) {
        loop {
            // Block until at least one result arrives
            match self.workers.recv_timeout(DEADLOCK_TIMEOUT) {
                Ok(first) => {
                    self.ingest(first);
                    // Drain all immediately available results
                    while let Some(result) = self.workers.try_recv() {
                        self.ingest(result);
                    }
                    if global::now() >= self.time_budget {
                        return;
                    }
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

    fn ingest(&mut self, mut task_result: TaskResult) {
        self.actors.submit(&mut task_result.events);
        self.done.push(Reverse(task_result.id));
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
            if global::now() == top.0.0 {
                return false;
            } else {
                global::fast_forward_clock(top.0.0);
                self.progress_bar.make_progress(top.0.0);
            }
        } else {
            if let Some(next_step_invocation_time) = self.actors.peek_next_step() {
                global::fast_forward_clock(next_step_invocation_time);
                self.progress_bar.make_progress(next_step_invocation_time);
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
                let task_id = self.workers.spawn_step(next_step);
                self.on_execution.push(Reverse(task_id));
            } else {
                break;
            }
        }
    }
}
