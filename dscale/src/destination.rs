use crate::Rank;

#[derive(Clone, Copy)]
pub(crate) enum Destination {
    BroadcastWithinPool(&'static str),
    Target(Rank),
}
