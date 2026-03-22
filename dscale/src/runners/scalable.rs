use std::{cmp::Reverse, collections::VecDeque, time::Duration};

use crossbeam_channel::RecvTimeoutError;

use crate::{
    actors::Actors,
    global::{self},
    global_unique_id,
    jiffy::Jiffies,
    now,
    runners::{
        SimulationRunner,
        emojis::{deadlock, looks_good},
        progress::Bar,
        task::{TaskId, TaskIndex, TaskResult},
        workers::Workers,
    },
    step::Step,
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
    /// Whether a process currently has a task executing in the thread pool.
    busy: Vec<bool>,
    /// Per-process queue of tasks within the window but deferred because the process is busy.
    waiting: Vec<VecDeque<(TaskId, Step)>>,
}

impl ScalableRunner {
    pub(crate) fn new(
        actors: Actors,
        time_budget: Jiffies,
        workers: Workers,
        safe_window: Jiffies,
    ) -> Self {
        let num_procs = workers.num_procs();
        Self {
            actors,
            time_budget,
            workers,
            progress_bar: Bar::new(time_budget),
            window_delta: safe_window,
            on_execution: TaskIndex::new(),
            done: TaskIndex::new(),
            busy: vec![false; num_procs],
            waiting: (0..num_procs).map(|_| VecDeque::new()).collect(),
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
        for rank in 0..self.workers.num_procs() {
            let step = Step::Start { rank };
            self.schedule(step);
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
        let rank = task_result.rank;
        self.actors.submit(&mut task_result.events);
        self.done.push(Reverse(task_result.id));

        // The process is no longer busy — check if there's a deferred task waiting.
        if let Some((waiting_id, waiting_step)) = self.waiting[rank].pop_front() {
            self.workers.spawn_step(waiting_id, waiting_step);
        } else {
            self.busy[rank] = false;
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
                self.schedule(next_step);
            } else {
                break;
            }
        }
    }

    /// Create a TaskId and either spawn immediately or defer if the target process is busy.
    fn schedule(&mut self, step: Step) {
        let task_id: TaskId = (global::now(), global_unique_id());
        let rank = step.target_rank();
        self.on_execution.push(Reverse(task_id));

        if self.busy[rank] {
            self.waiting[rank].push_back((task_id, step));
        } else {
            self.busy[rank] = true;
            self.workers.spawn_step(task_id, step);
        }
    }
}
