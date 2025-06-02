use crate::ocr::{agents::Agent, frontier::Frontier, timer::Timer};

pub struct MatchResult {
    pub agents: Vec<Option<Agent>>,
    pub timer: Timer,
    pub restart_amount: u8,
    pub frontier: Frontier,
}
