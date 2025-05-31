use crate::{agents::Agent, timer::Timer};

pub struct MatchResult {
    pub agents: Vec<Option<Agent>>,
    pub timer: Timer,
}
