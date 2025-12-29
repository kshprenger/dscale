use std::{cell::RefCell, process::exit, rc::Rc};

use log::{error, info};

use crate::{
    access,
    actor::SharedActor,
    network::{BandwidthType, Network},
    process::{ProcessId, ProcessPool, UniqueProcessHandle},
    progress::Bar,
    random::{self},
    time::{FastForwardClock, Jiffies, Now, timer::Timers},
};

pub struct Simulation {
    actors: Vec<SharedActor>,
    max_time: Jiffies,
    progress_bar: Bar,
}

const K_PROGRESS_TIMES: usize = 10;

impl Simulation {
    pub(crate) fn New(
        seed: random::Seed,
        max_time: Jiffies,
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        procs: Vec<(ProcessId, UniqueProcessHandle)>,
    ) -> Self {
        let _ = env_logger::try_init();

        let proc_pool = ProcessPool::NewShared(procs);

        let mut actors = Vec::new();

        let network_actor = Rc::new(RefCell::new(Network::New(
            seed,
            max_network_latency,
            bandwidth_type,
            proc_pool.clone(),
        )));

        let timers_actor = Rc::new(RefCell::new(Timers::New(proc_pool.clone())));

        access::SetupAccess(network_actor.clone(), timers_actor.clone());

        actors.push(network_actor as SharedActor);
        actors.push(timers_actor as SharedActor);

        Self {
            actors,
            max_time,
            progress_bar: Bar::New(max_time, max_time.0 / K_PROGRESS_TIMES),
        }
    }

    pub fn Run(&mut self) {
        self.Start();

        while self.KeepRunning() {
            match self.PeekClosest() {
                None => {
                    error!("DEADLOCK! (ﾉಥ益ಥ）ﾉ ┻━┻ Try with RUST_LOG=debug");
                    exit(1)
                }
                Some((time, actor)) => {
                    FastForwardClock(time);
                    actor.borrow_mut().Step();
                    access::Drain(); // Only after Step() to avoid double borrow_mut() of SharedActor
                    self.progress_bar.MakeProgress(Now());
                }
            }
        }

        info!("Looks good! ヽ(‘ー`)ノ");
    }
}

impl Simulation {
    fn KeepRunning(&mut self) -> bool {
        Now() < self.max_time
    }

    fn Start(&mut self) {
        self.actors.iter_mut().for_each(|actor| {
            actor.borrow_mut().Start();
            access::Drain(); // Only after Start() to avoid double borrow_mut() of SharedActor
        });
    }

    // O(n) -> O(log(n))?
    fn PeekClosest(&mut self) -> Option<(Jiffies, SharedActor)> {
        self.actors
            .iter_mut()
            .filter_map(|actor| Some((actor.borrow().PeekClosest()?, actor.clone())))
            .min_by_key(|tuple| tuple.0)
    }
}
