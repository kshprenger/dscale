use std::{cmp::Reverse, collections::VecDeque};

use crossbeam_channel::RecvError;

use crate::{
    actors::Actors,
    global::{self},
    global_unique_id,
    jiffy::Jiffies,
    now,
    runners::{
        RunStatus, SimulationRunner,
        progress::Bar,
        task::{TaskId, TaskIndex, TaskResult},
        workers::Workers,
    },
    step::Step,
};

pub(crate) struct ScalableRunner {
    actors: Actors,
    time_budget: Jiffies,
    workers: Workers,
    progress_bar: Bar,
    window_delta: Jiffies,
    on_execution: TaskIndex,
    done: TaskIndex,
    // Whether a process currently has a task executing in the thread pool.
    busy: Vec<bool>,
    // Per-process queue of tasks within the window but deferred because the process is busy.
    // Keeps sequential order per process within window
    waiting: Vec<VecDeque<(TaskId, Step)>>,
    started: bool,
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
            started: false,
        }
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.started = true;
            for rank in 0..self.workers.num_procs() {
                let step = Step::Start { rank };
                let task_id: TaskId = (global::now(), global_unique_id());
                self.workers.install_step(task_id, step);
                self.busy[rank] = true;
                self.on_execution.push(Reverse(task_id));
            }
        }
    }
}

impl Drop for ScalableRunner {
    fn drop(&mut self) {
        global::reset();
    }
}

impl SimulationRunner for ScalableRunner {
    fn run_full_budget(&mut self) -> RunStatus {
        self.ensure_started();
        let status = self.coordinate(None, self.time_budget);
        self.join_workers();
        self.progress_bar.finish();
        status
    }

    fn run_steps(&mut self, k: usize) -> RunStatus {
        self.ensure_started();
        let status = self.coordinate(Some(k), self.time_budget);
        self.join_workers();
        status
    }

    fn run_sub_budget(&mut self, sub_budget: Jiffies) -> RunStatus {
        self.ensure_started();
        let deadline = std::cmp::min(now() + sub_budget, self.time_budget);
        let status = self.coordinate(None, deadline);
        self.join_workers();
        status
    }
}

impl ScalableRunner {
    /// Coordinate the worker pool.
    /// - `max_steps`: if `Some(k)`, stop after `k` ingested results.
    /// - `deadline`: stop when simulation time reaches this value.
    fn coordinate(&mut self, max_steps: Option<usize>, deadline: Jiffies) -> RunStatus {
        let mut steps: usize = 0;
        loop {
            if let Some(k) = max_steps {
                if steps >= k {
                    return RunStatus::Completed { steps };
                }
            }

            // Block until at least one result arrives
            match self.workers.next_result() {
                Ok(first) => {
                    self.ingest(first);
                    steps += 1;

                    if global::now() >= deadline {
                        if global::now() >= self.time_budget {
                            return RunStatus::BudgetExhausted { steps };
                        }
                        return RunStatus::Completed { steps };
                    }

                    if let Some(k) = max_steps {
                        if steps >= k {
                            return RunStatus::Completed { steps };
                        }
                    }

                    // Drain all immediately available results
                    while let Some(result) = self.workers.try_next_result() {
                        self.ingest(result);
                        steps += 1;
                        if let Some(k) = max_steps {
                            if steps >= k {
                                return RunStatus::Completed { steps };
                            }
                        }
                    }

                    self.adjust_task_index();
                    self.try_advance();
                }
                Err(RecvError) => {
                    unreachable!("unexpected worker disconnection")
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
                // There is still some top task executing in window — move to this task
                global::fast_forward_clock(top.0.0);
                self.progress_bar.make_progress(top.0.0);
            }
        } else {
            // No tasks in window — try to find new next task outside window
            if let Some(next_step_invocation_time) = self.actors.peek_next_step() {
                global::fast_forward_clock(next_step_invocation_time);
                self.progress_bar.make_progress(next_step_invocation_time);
            } else {
                // No more events — quiesced. Not a deadlock, coordinate will
                // exit on the next iteration when it blocks on next_result and
                // no workers are busy, or the caller checks the step limit.
                return false;
            }
        }
        true
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

    fn join_workers(&mut self) {
        for queue in &mut self.waiting {
            queue.clear();
        }
        while self.busy.iter().any(|&b| b) {
            match self.workers.next_result() {
                Ok(result) => {
                    self.busy[result.rank] = false;
                }
                Err(RecvError) => {
                    unreachable!("unexpected worker disconnection")
                }
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
