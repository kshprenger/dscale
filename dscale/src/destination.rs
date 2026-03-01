use crate::Rank;

pub enum Destination {
    BroadcastWithinPool(&'static str),
    To(Rank),
}
