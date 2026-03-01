//! Simulation engine and execution control.
//!
//! This module contains the core simulation engine that drives the event loop
//! and manages the execution of distributed system simulations. The `Simulation`
//! struct orchestrates all simulation actors including network, timers, and
//! process execution in a deterministic, single-threaded environment.

use std::{cell::RefCell, process::exit, rc::Rc, usize};

use log::{error, info};

use crate::{
    actor::SharedActor,
    global,
    network::{BandwidthDescription, Network},
    nursery::{HandlerMap, Nursery},
    progress::Bar,
    random::{self, Randomizer},
    time::{Jiffies, timer_manager::TimerManager},
    topology::{LatencyTopology, PoolListing, Topology},
};

/// The main simulation engine that executes distributed system simulations.
///
/// `Simulation` is the core engine that drives a DScale simulation. It manages
/// the event loop, coordinates between different simulation actors (network,
/// timers, processes), and ensures deterministic execution through careful
/// event scheduling.
///
/// The simulation runs in a single thread using an event-driven architecture
/// where all actions are scheduled as discrete events in simulation time.
/// This approach ensures deterministic behavior and makes it possible to
/// reproduce exact simulation runs.
///
/// # Architecture
///
/// The simulation consists of several key components:
/// - **Network Actor**: Handles message routing and bandwidth simulation
/// - **Timer Manager**: Manages scheduled timers for processes
/// - **Process Nursery**: Manages process lifecycle and message delivery
/// - **Global State**: Provides access to simulation-wide services
///
/// # Lifecycle
///
/// 1. **Initialization**: Set up actors and global state
/// 2. **Start Phase**: Call `start()` on all processes
/// 3. **Event Loop**: Process events in chronological order until time budget or deadlock
/// 4. **Cleanup**: Reset global state and complete execution
///
/// #[derive(Default)]
/// struct MyProcess;
///
/// impl ProcessHandle for MyProcess {
///     fn start(&mut self) {
///         // Process initialization
///     }
///
///     fn on_message(&mut self, from: Rank, message: MessagePtr) {
///         // Handle incoming messages
///     }
///
///     fn on_timer(&mut self, id: TimerId) {
///         // Handle timer events
///     }
/// }
///
/// let mut simulation = SimulationBuilder::default()
///     .add_pool::<MyProcess>("nodes", 5)
///     .time_budget(Jiffies(100_000))
///     .build();
///
/// simulation.run(); // Execute the simulation
/// ```
///
/// [`SimulationBuilder`]: crate::SimulationBuilder
pub struct Simulation {
    actors: Vec<SharedActor>,
    time_budget: Jiffies,
    progress_bar: Bar,
}

impl Simulation {
    pub(crate) fn new(
        seed: random::Seed,
        time_budget: Jiffies,
        bandwidth: BandwidthDescription,
        latency_topology: LatencyTopology,
        pool_listing: PoolListing,
        procs: HandlerMap,
    ) -> Self {
        let topology = Topology::new_shared(pool_listing.clone(), latency_topology);
        let nursery = Nursery::new(procs);

        let network_actor = Rc::new(RefCell::new(Network::new(
            seed,
            bandwidth,
            topology.clone(),
            nursery.clone(),
        )));

        let timers_actor = Rc::new(RefCell::new(TimerManager::new(nursery.clone())));

        global::configuration::setup_global_configuration(nursery.size());
        global::setup_access(
            network_actor.clone(),
            timers_actor.clone(),
            topology,
            Randomizer::new(seed),
        );

        let actors: Vec<SharedActor> = vec![network_actor, timers_actor];

        Self {
            actors,
            time_budget,
            progress_bar: Bar::new(time_budget),
        }
    }

    /// Executes the simulation until completion.
    ///
    /// This method runs the main simulation loop, processing events in chronological
    /// order until either the time budget is exhausted or a deadlock occurs (no more
    /// events to process). The simulation follows these phases:
    ///
    /// 1. **Start Phase**: Calls `start()` on all processes to initialize them
    /// 2. **Event Loop**: Processes events in time order, advancing the simulation clock
    /// 3. **Completion**: Finishes when time budget is reached or no events remain
    ///
    /// # Event Processing
    ///
    /// During each simulation step:
    /// - The earliest scheduled event is identified across all actors
    /// - The simulation clock advances to that event's time
    /// - The appropriate actor processes the event
    /// - New events may be scheduled as a result
    /// - Progress is reported for long-running simulations
    ///
    /// # Termination Conditions
    ///
    /// The simulation terminates when:
    /// - **Time Budget Exhausted**: The simulation reaches its configured time limit
    /// - **Deadlock Detected**: No more events are scheduled (may indicate a bug)
    ///
    /// # Error Handling
    ///
    /// If a deadlock is detected (no events remaining before time budget), the
    /// simulation will log an error and exit. This typically indicates a bug in
    /// the process logic where processes fail to schedule continuing work.
    ///
    /// # struct MyProcess;
    /// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
    /// # impl dscale::ProcessHandle for MyProcess {
    /// #     fn start(&mut self) {}
    /// #     fn on_message(&mut self, from: dscale::Rank, message: dscale::MessagePtr) {}
    /// #     fn on_timer(&mut self, id: dscale::TimerId) {}
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This method will cause the program to exit with an error code if a deadlock
    /// is detected. Use `RUST_LOG=debug` for detailed information about the
    /// deadlock condition.
    pub fn run(&mut self) {
        self.start();

        while global::now() < self.time_budget {
            self.step();
        }

        // For small simulations progress bar is not fullfilling
        self.progress_bar.finish();

        info!("Looks good! ヽ('ー`)ノ");
    }
}

impl Simulation {
    fn start(&mut self) {
        self.actors.iter_mut().for_each(|actor| {
            actor.borrow_mut().start();
            global::schedule(); // Only after start() to avoid double borrow_mut() of SharedActor
        });
    }

    fn step(&mut self) {
        match self.peek_closest() {
            None => {
                error!("DEADLOCK! (ﾉಥ益ಥ）ﾉ ┻━┻ Try with RUST_LOG=debug");
                exit(1)
            }
            Some((future, actor)) => {
                global::fast_forward_clock(future);
                actor.borrow_mut().step();
                global::schedule(); // Only after step() to avoid double borrow_mut() of SharedActor
                self.progress_bar
                    .make_progress(future.min(self.time_budget));
            }
        }
    }

    fn peek_closest(&mut self) -> Option<(Jiffies, SharedActor)> {
        let mut min_time = Jiffies(usize::MAX);
        let mut sha: Option<SharedActor> = None;
        for actor in self.actors.iter() {
            actor.borrow().peek_closest().map(|time| {
                if time < min_time {
                    min_time = time;
                    sha = Some(actor.clone())
                }
            });
        }

        Some((min_time, sha?))
    }
}

impl Drop for Simulation {
    fn drop(&mut self) {
        global::drop_all(); // Clear thread_locals
    }
}
