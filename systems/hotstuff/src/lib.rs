// https://arxiv.org/pdf/1803.05069

use std::{collections::HashMap, rc::Rc};

use dscale::{
    global::{anykv, configuration::process_number},
    helpers::Combiner,
    *,
};

pub const B0: &str = "b_0";
pub const HOTSTUFF_POOL: &str = "HotstuffCluster";

type NodeId = usize;

pub struct Node {
    pub id: NodeId,
    pub parent: Option<Rc<Node>>,
    pub height: usize,
    pub creator: Rank,
    pub creation_time: Jiffies,
}

pub enum HSMessage {
    Propose(Rc<Node>),
    Vote(Rc<Node>),
}

impl Message for HSMessage {}

pub struct ChainedHotstuff {
    pending_quorums: HashMap<NodeId, Combiner<()>>,
    vheight: usize,
    b_lock: Rc<Node>,
    b_exec: Rc<Node>,
    b_leaf: Rc<Node>,
}

impl ProcessHandle for ChainedHotstuff {
    fn start(&mut self) {
        if rank() == 1 {
            broadcast(HSMessage::Propose(self.create_leaf()));
        }
    }
    fn on_message(&mut self, from: Rank, message: MessagePtr) {
        debug_process!("Got message from {from}");
        match message.as_type::<HSMessage>().as_ref() {
            HSMessage::Propose(b_new) => {
                if b_new.height > self.vheight
                    && (self.extends(b_new) || b_new.height > self.b_lock.height)
                {
                    self.vheight = b_new.height;
                    send_to(self.get_next_leader(), HSMessage::Vote(b_new.clone()));
                    self.update(b_new.clone());
                } else {
                    debug_process!("Prososal rejected")
                }
            }
            HSMessage::Vote(node) => {
                let combiner_size = self.quorum_size();
                let combiner = self
                    .pending_quorums
                    .entry(node.id)
                    .or_insert(Combiner::<()>::new(combiner_size));

                if let Some(_) = combiner.combine(()) {
                    self.b_leaf = node.clone();
                    debug_process!("Quorum gathered!");
                    broadcast(HSMessage::Propose(self.create_leaf()));
                }
            }
        }
    }
    fn on_timer(&mut self, _id: dscale::TimerId) {
        unreachable!()
    }
}

impl Default for ChainedHotstuff {
    fn default() -> Self {
        let genesis_node = anykv::get::<Rc<Node>>(B0);
        Self {
            pending_quorums: HashMap::new(),
            vheight: 0,
            b_lock: genesis_node.clone(),
            b_exec: genesis_node.clone(),
            b_leaf: genesis_node.clone(),
        }
    }
}

// Internals
impl ChainedHotstuff {
    fn update(&mut self, b_star: Rc<Node>) {
        let b__ = b_star.parent.clone();
        let b_ = b__.map(|b__| b__.parent.clone()).flatten();
        let b = b_.clone().map(|b_| b_.parent.clone()).flatten();

        if b_star.height > self.b_leaf.height {
            self.b_leaf = b_star
        }
        if let Some(ref b_) = b_ {
            if b_.height > self.b_lock.height {
                self.b_lock = b_.clone();
            }
        }

        if let Some(b) = b {
            self.on_commit(b.clone());
            self.b_exec = b;
        }
    }

    fn on_commit(&mut self, b: Rc<Node>) {
        if self.b_exec.height < b.height {
            self.on_commit(b.parent.clone().unwrap());
            if rank() == b.creator {
                anykv::modify::<(f64, usize)>(
                    "avg_latency",
                    |(prev_avg_latency, prev_total_ordered)| {
                        let vertex_latency = now() - b.creation_time;
                        *prev_avg_latency = (vertex_latency.0 as f64
                            + (*prev_avg_latency * *prev_total_ordered as f64))
                            as f64
                            / (*prev_total_ordered + 1) as f64;

                        *prev_total_ordered += 1;
                    },
                );
            }
        }
    }
}

// Utils
impl ChainedHotstuff {
    fn create_leaf(&self) -> Rc<Node> {
        let parent = self.b_leaf.clone();
        Rc::new(Node {
            id: global_unique_id(),
            parent: Some(parent.clone()),
            height: parent.height + 1,
            creator: rank(),
            creation_time: now(),
        })
    }

    fn get_next_leader(&self) -> Rank {
        (self.vheight % process_number()) + 1
    }

    fn quorum_size(&self) -> usize {
        (process_number() * 2) / 3 + 1
    }

    fn extends(&self, child: &Rc<Node>) -> bool {
        Rc::ptr_eq(child.parent.as_ref().unwrap(), &self.b_lock)
    }
}
