use super::{match_result::MatchResult, GameMatch, GameState, HalfStage, PlayerAction, Stage};

// References are mutable to change the state of
// self.game.tick
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
            if self.game.visibility_flags.challenges() || self.game.visibility_flags.hp() {
                Some(Stage::FirstHalf(HalfStage::Prepare))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn transition_from_first_half(&mut self) -> Option<Stage> {
        if let Stage::FirstHalf(half_stage) = &self.game.stage {
            // if self.game.visibility_flags.hp() {
            //     self.player_state = PlayerAction::None;
            // }
            match self.player_state {
                PlayerAction::RestartDialog => {
                    if !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                        && self.game.visibility_flags.loading()
                    {
                        if let HalfStage::Run | HalfStage::Cleared = half_stage {
                            self.game.restart_amount += 1;
                        }

                        self.player_state = PlayerAction::None;
                        return Some(Stage::FirstHalf(HalfStage::Prepare));
                    }
                }
                PlayerAction::ExitDialog => {
                    if !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                        && self.game.visibility_flags.loading()
                    {
                        // if let HalfStage::Run | HalfStage::Cleared = half_stage {
                        //     self.game.restart_amount += 1;
                        // }

                        self.player_state = PlayerAction::None;
                        println!("Leaving the game");
                        return Some(Stage::Pick);
                    }
                }
                _ => {}
            }

            if let PlayerAction::RestartDialog = self.player_state {};

            if PlayerAction::None != self.player_state {
                return None;
            }

            match half_stage {
                HalfStage::Prepare => {
                    self.game.is_dirty = false;

                    if self.game.ingame_timer.is_some()
                        && self.game.visibility_flags.hp()
                        && self.game.visibility_flags.ingame_timer()
                    {
                        Some(Stage::FirstHalf(HalfStage::Run))
                    } else {
                        None
                    }
                }
                HalfStage::Run => {
                    self.game.is_dirty = true;
                    if self.game.visibility_flags.blackscreen() {
                        self.game.visibility_flags.set_blackscreen(false);
                        return Some(Stage::SecondHalf(HalfStage::Prepare));
                    }

                    if self.game.ingame_timer.is_some()
                        && self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                    {
                        // NOTE: This delay is probably too short but idk
                        if self.game.tick >= 1 {
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
            match self.player_state {
                PlayerAction::RestartDialog => {
                    if !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                        && self.game.visibility_flags.loading()
                    {
                        if let HalfStage::Run | HalfStage::Cleared = half_stage {
                            self.game.restart_amount += 1;
                        }

                        self.player_state = PlayerAction::None;
                        return Some(Stage::SecondHalf(HalfStage::Prepare));
                    }
                }
                PlayerAction::ExitDialog => {
                    if !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                        && self.game.visibility_flags.loading()
                    {
                        // if let HalfStage::Run | HalfStage::Cleared = half_stage {
                        //     self.game.restart_amount += 1;
                        // }

                        self.player_state = PlayerAction::None;
                        println!("Leaving the game");
                        return Some(Stage::Pick);
                    }
                }
                _ => {}
            }

            if PlayerAction::None != self.player_state {
                return None;
            }

            match half_stage {
                HalfStage::Prepare => {
                    self.game.is_dirty = false;

                    if self.game.ingame_timer.is_some()
                        && self.game.visibility_flags.hp()
                        && self.game.visibility_flags.ingame_timer()
                    {
                        Some(Stage::SecondHalf(HalfStage::Run))
                    } else {
                        None
                    }
                }
                HalfStage::Run => {
                    self.game.is_dirty = true;
                    if self.game.ingame_timer.is_some()
                        && !self.game.visibility_flags.hp()
                        && !self.game.visibility_flags.ingame_timer()
                        && self.game.visibility_flags.res_timer()
                    {
                        Some(Stage::SecondHalf(HalfStage::Cleared))
                    } else {
                        None
                    }
                }
                // Unneccesary handling but let it be
                HalfStage::Cleared => {
                    println!("now im looping here");
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
                .expect("expect game.frontier to be Some"),

            agents: self
                .game
                .agents
                .take()
                .expect("expect game.agents to be Some"),

            timer: self
                .game
                .res_timer
                .take()
                .expect("expect game.timer to be Some"),

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

#[cfg(test)]
mod tests {
    use crate::ocr::is_black_screen;

    #[test]
    fn black() {
        let img = image::open("black_screen.png").unwrap();
        let img = img.to_rgba8();

        let res = is_black_screen(&img);
        println!("{res}");
    }
}
