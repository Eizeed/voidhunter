use super::{match_result::MatchResult, GameMatch, GameState, HalfStage, PlayerAction, Stage};

// References are mutable to change the state of
// self.game.next_stage and self.game.tick
impl GameMatch {
    pub fn transition(&mut self) -> Option<Stage> {
        match self.game.stage {
            Stage::Pick => self.transition_from_pick(),
            Stage::FirstHalf(_) => self.transition_from_first_half(),
            Stage::SecondHalf(_) => self.transition_from_second_half(),
            Stage::Finished => self.transition_from_finished(),
            _ => None,
        }
    }

    pub fn transition_from_pick(&mut self) -> Option<Stage> {
        if self.game.agents.is_some()
            && self.game.frontier.is_some()
            && !self.game.visibility_flags.frontier()
            && !self.game.visibility_flags.agents()
        {
            if self.game.next_stage
                && (self.game.visibility_flags.challenges() || self.game.visibility_flags.hp())
            {
                Some(Stage::FirstHalf(HalfStage::Prepare))
            } else {
                self.game.next_stage = true;
                None
            }
        } else {
            self.game.next_stage = false;
            None
        }
    }

    pub fn transition_from_first_half(&mut self) -> Option<Stage> {
        if let Stage::FirstHalf(half_stage) = &self.game.stage {
            // if self.game.visibility_flags.hp() {
            //     self.player_state = PlayerAction::None;
            // }

            if let PlayerAction::RestartDialog = self.player_state {
                if self.game.is_dirty
                    && !self.game.visibility_flags.hp()
                    && !self.game.visibility_flags.ingame_timer()
                    && self.game.visibility_flags.loading()
                {
                    if let HalfStage::Run | HalfStage::Cleared = half_stage {
                        self.game.restart_amount += 1;
                    }

                    self.player_state = PlayerAction::None;
                    return Some(Stage::FirstHalf(HalfStage::Prepare));
                }
            };
            
            if PlayerAction::None != self.player_state {
                return None;
            }

            match half_stage {
                HalfStage::Prepare => {
                    self.game.is_dirty = false;
                    self.game.next_stage = true;

                    if self.game.ingame_timer.is_some()
                        && self.game.visibility_flags.hp()
                        && self.game.visibility_flags.ingame_timer()
                    {
                        Some(Stage::FirstHalf(HalfStage::Run))
                    } else {
                        None
                    }
                }
                // TODO: Create a mechanism to track if user is in pause and
                // don't change if he is
                HalfStage::Run => {
                    self.game.is_dirty = true;

                    if self.game.ingame_timer.is_some()
                        && self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                    {
                        if self.game.tick >= 2 {
                            self.game.tick = 0;
                            println!("Time to change");
                            Some(Stage::FirstHalf(HalfStage::Cleared))
                        } else {
                            println!("Ticking");
                            self.game.tick += 1;
                            None
                        }
                    } else {
                        self.game.tick = 0;
                        None
                    }
                }
                // TODO: Create a mechanism to track if user is in pause and
                // don't change if he is
                HalfStage::Cleared => {
                    if self.game.ingame_timer.is_some()
                        && !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                    {
                        Some(Stage::SecondHalf(HalfStage::Prepare))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn transition_from_second_half(&mut self) -> Option<Stage> {
        if let Stage::SecondHalf(half_stage) = &self.game.stage {
            // if self.game.visibility_flags.hp() {
            //     self.player_state = PlayerAction::None;
            // }

            if let PlayerAction::RestartDialog = self.player_state {
                if self.game.is_dirty
                    && !self.game.visibility_flags.hp()
                    && !self.game.visibility_flags.ingame_timer()
                    && self.game.visibility_flags.loading()
                {
                    if let HalfStage::Run | HalfStage::Cleared = half_stage {
                        self.game.restart_amount += 1;
                    }

                    self.player_state = PlayerAction::None;
                    return Some(Stage::SecondHalf(HalfStage::Prepare));
                }
            };

            if PlayerAction::None != self.player_state {
                return None;
            }

            match half_stage {
                HalfStage::Prepare => {
                    self.game.is_dirty = false;
                    self.game.next_stage = true;

                    if self.game.ingame_timer.is_some()
                        && self.game.visibility_flags.hp()
                        && self.game.visibility_flags.ingame_timer()
                    {
                        Some(Stage::SecondHalf(HalfStage::Run))
                    } else {
                        None
                    }
                }
                // TODO: Create a mechanism to track if user is in pause and
                // don't change if he is
                HalfStage::Run => {
                    self.game.is_dirty = true;
                    if self.game.ingame_timer.is_some()
                        && !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                    {
                        // Need to check if it's not just a chain of events
                        // where hp and timer isn't present.
                        // Also in second half there is no state with only hp
                        // visible.
                        if self.game.tick >= 18 {
                            println!("Time to change");
                            self.game.tick = 0;
                            Some(Stage::SecondHalf(HalfStage::Cleared))
                        } else {
                            println!("Ticking");
                            self.game.tick += 1;
                            None
                        }
                    } else {
                        self.game.tick = 0;
                        None
                    }
                }
                // TODO: Create a mechanism to track if user is in pause and
                // don't change if he is
                HalfStage::Cleared => {
                    if self.game.ingame_timer.is_some()
                        && self.game.res_timer.is_some()
                        && self.game.visibility_flags.res_timer()
                        && !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                    {
                        Some(Stage::Finished)
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }
    pub fn transition_from_finished(&mut self) -> Option<Stage> {
        // println!("Chaged state to finished:");
        // println!("frontier: {:?}", self.game.frontier);
        // println!("agents: {:?}", self.game.agents);
        // println!("timer: {:?}", self.game.res_timer);
        // println!("restarts: {:?}", self.game.restart_amount);
        let match_res = MatchResult {
            frontier: self
                .game
                .frontier
                .take()
                .expect("Expect game.frontier to be Some"),

            agents: self
                .game
                .agents
                .take()
                .expect("Expect game.agents to be Some"),

            timer: self
                .game
                .res_timer
                .take()
                .expect("Expect game.timer to be Some"),

            restart_amount: self.game.restart_amount,
        };

        self.match_results.push(match_res);

        if self.match_results.len() < 2 {
            self.game = GameState::new();
            Some(Stage::Pick)
        } else {
            Some(Stage::GameOver)
        }
    }
}
