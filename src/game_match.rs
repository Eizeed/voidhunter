use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use iced::{
    alignment::Horizontal,
    time,
    widget::{button, column, text, Column, Row},
    Color, Element, Length, Subscription, Task,
};
use image::RgbaImage;
use match_result::MatchResult;
use tokio::task::spawn_blocking;

use crate::{
    agents::{Agent, PickStage},
    capture,
    timer::{RunStage, Timer, TimerStage},
};

pub enum Action {
    Run(Task<Message>),
    Home,
    None,
}

mod match_result;

#[derive(Debug, Clone)]
pub enum Message {
    Home,

    ScanTick(Instant),
    SetAgents(Option<Vec<Option<Agent>>>),
    SetTimer(Option<Timer>),
    SetIngameTimer(Option<Timer>),
    ChangeStage(Stage),
}

#[derive(Debug, Clone)]
pub enum Stage {
    Pick,
    Run,
    Timer,
    Finished,
    GameOver,
}

pub struct GameMatch {
    timer: Option<Timer>,
    ingame_timer: Option<Timer>,
    agents: Option<Vec<Option<Agent>>>,
    current_image: Arc<Mutex<Vec<u8>>>,
    window_exists: bool,
    stage: Stage,
    match_result: Vec<MatchResult>,
    prepare_next_stage: bool,
}

impl GameMatch {
    pub fn new() -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let window_exists = capture(buffer.clone()).is_ok();

        GameMatch {
            timer: None,
            ingame_timer: None,
            agents: None,
            current_image: buffer,
            window_exists,
            stage: Stage::Pick,
            match_result: Vec::with_capacity(2),
            prepare_next_stage: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        let task = match message {
            Message::Home => Action::Home,
            Message::ScanTick(_now) => {
                println!("500 ms passed...");
                println!("Scanning");
                println!("");

                if !self.window_exists {
                    let res = capture(self.current_image.clone());
                    if res.is_ok() {
                        self.window_exists = true;
                    }

                    return Action::None;
                }

                let image_buf = self.current_image.lock().unwrap().clone();
                if image_buf.is_empty() {
                    return Action::None;
                }
                let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();

                let prepare_next_stage = self.prepare_next_stage.clone();
                match self.stage {
                    Stage::Pick => Action::Run(Task::future(async move {
                        let run_timer = if prepare_next_stage {
                            let ocr = RunStage::get_timer_ocr(&mut image);
                            Timer::from_raw_ocr(ocr.as_str())
                        } else {
                            None
                        };

                        if run_timer.is_some() {
                            return Message::ChangeStage(Stage::Run);
                        }

                        let agents = spawn_blocking(move || {
                            let ocr = PickStage::get_agent_ocr(&mut image);
                            Agent::from_raw_ocr(&ocr)
                        })
                        .await
                        .unwrap();

                        Message::SetAgents(agents)
                    })),
                    Stage::Run => Action::Run(Task::future(async move {
                        let res_timer = if prepare_next_stage {
                            let ocr = TimerStage::get_timer_ocr(&mut image);
                            Timer::from_raw_ocr(ocr.as_str())
                        } else {
                            None
                        };

                        if let Some(timer) = res_timer {
                            return Message::SetTimer(Some(timer));
                        }

                        let ocr = RunStage::get_timer_ocr(&mut image);
                        let timer = Timer::from_raw_ocr(ocr.as_str());

                        Message::SetIngameTimer(timer)
                    })),
                    Stage::Timer => Action::Run(Task::future(async {
                        let timer = spawn_blocking(move || {
                            let ocr = TimerStage::get_timer_ocr(&mut image);
                            Timer::from_raw_ocr(ocr.as_str())
                        })
                        .await
                        .unwrap();

                        Message::SetTimer(timer)
                    })),
                    _ => Action::None,
                }
            }
            Message::SetAgents(agents) => {
                if self.agents.is_some() && agents.is_none() {
                    self.prepare_next_stage = true;
                } else {
                    self.prepare_next_stage = false;
                    self.agents = agents;
                }

                Action::None
            }
            Message::SetIngameTimer(timer) => {
                if self.ingame_timer.is_some() && timer.is_none() {
                    self.prepare_next_stage = true;
                } else {
                    self.prepare_next_stage = false;
                    self.ingame_timer = timer;
                }

                Action::None
            }
            Message::SetTimer(timer) => {
                if self.timer.is_some() {
                    Action::Run(Task::done(Message::ChangeStage(Stage::Finished)))
                } else {
                    self.timer = timer;
                    Action::None
                }
            }
            Message::ChangeStage(stage) => {
                self.prepare_next_stage = false;
                self.stage = stage;

                match &self.stage {
                    Stage::Finished => {
                        let result = MatchResult {
                            agents: self.agents.take().expect("Expect agents is always Some"),
                            timer: self.timer.take().expect("expect timer to be Some"),
                        };

                        self.match_result.push(result);

                        if self.match_result.len() == 1 {
                            self.stage = Stage::GameOver;
                        } else {
                            self.stage = Stage::Pick;
                        }
                    }
                    _ => {}
                }
                Action::None
            }
        };

        task
    }

    pub fn view(&self) -> Element<Message> {
        if !self.window_exists {
            return text("Zenless zone zero not found. Waiting for start of the game")
                .size(20)
                .into();
        }

        let col_content = Column::new();
        let current_stage = text(format!("{:?}", self.stage))
            .size(25)
            .color(Color::WHITE);

        let col_content = col_content.push(current_stage);

        let change_stage = text(format!("Change state: {:?}", self.prepare_next_stage))
            .size(20)
            .color(Color::WHITE);

        let col_content = col_content.push(change_stage);

        // let col_content = col_content.push(text(format!(
        //     "there are {} match results",
        //     self.match_result.len()
        // )));

        let col_content = col_content.push(match self.stage {
            Stage::GameOver => {
                let mut iter = self
                    .match_result
                    .iter()
                    .map(|r| (&r.agents, &r.timer))
                    .enumerate();

                let mut cols = Vec::with_capacity(2);

                while let Some((idx, (agents, timer))) = iter.next() {
                    let header = text(format!("Roster {}", idx + 1))
                        .size(20)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill);

                    let timer = text(timer.to_string())
                        .size(20)
                        .color(Color::WHITE)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill);

                    let agents = Self::agents(agents.as_slice());

                    cols.push(column![header, column![timer, agents]].spacing(20).into());
                }

                Column::from_vec(cols).width(Length::Fill).spacing(30)
            }
            _ => {
                let timer = if let Some(timer) = &self.ingame_timer {
                    text(format!("Ingame timer: {}", timer.to_string()))
                        .size(20)
                        .color(Color::WHITE)
                } else {
                    text("No timer on the screen").size(20).color(Color::WHITE)
                };

                let agents: Element<_, _, _> = match self.agents.as_ref() {
                    Some(agents) => {
                        let header = text("Chosen agents:").size(20).color(Color::WHITE);
                        let agents = Self::agents(agents.as_slice());
                        column![header, agents].into()
                    }
                    None => text("Not in Pick Stage").into(),
                };

                column![timer, agents].width(Length::Fill)
            }
        });

        let col_content = col_content.push(button("Home").on_press(Message::Home));

        col_content.spacing(20).into()
    }

    pub fn subscribtion(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(500)).map(Message::ScanTick)
    }

    fn agents(agents: &[Option<Agent>]) -> Element<Message> {
        let (first_team, second_team) = agents.split_at(3);

        let row_1 = Row::from_iter(first_team.iter().map(|c| {
            let name = c.as_ref().map(|c| c.name.as_str()).unwrap_or("EMPTY");
            text(name)
                .size(20)
                .color(Color::WHITE)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .into()
        }))
        .padding([0, 20]);

        let row_2 = Row::from_iter(second_team.iter().map(|c| {
            let name = c.as_ref().map(|c| c.name.as_str()).unwrap_or("EMPTY");
            text(name)
                .size(20)
                .color(Color::WHITE)
                .align_x(Horizontal::Center)
                .width(Length::Fill)
                .into()
        }))
        .padding([0, 20]);

        column![row_1, row_2].width(Length::Fill).into()
    }
}
