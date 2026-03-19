use crate::Rank;

#[derive(Clone, Copy)]
pub enum Destination {
    BroadcastWithinPool(&'static str),
    To(Rank),
}
