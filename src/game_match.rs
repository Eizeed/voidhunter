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
    capture,
    ocr::{
        agents::{Agent, PickStage},
        confirm::{ConfirmDialog, ConfirmOcr},
        pause::{Pause, PauseOcr},
        timer::{RunStage, Timer, TimerStage},
    },
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
    SetIngameTimer(Option<Timer>),
    SetTimer(Timer),
    ChangeStage(Stage),
    SetRestart(bool, bool),
    // SetPause(bool),
    // SetConfirm(bool),
    None,
}

#[derive(Debug, Clone)]
pub enum Stage {
    Pick,
    Run,
    Finished,
    GameOver,
}

pub struct GameMatch {
    timer: Option<Timer>,
    ingame_timer: Option<Timer>,
    restart_amount: u8,
    agents: Option<Vec<Option<Agent>>>,
    current_image: Arc<Mutex<Vec<u8>>>,
    window_exists: bool,
    stage: Stage,
    match_result: Vec<MatchResult>,
    prepare_next_stage: bool,
    in_pause: bool,
    count_restart: bool,
    pause_tick_counter: u32,
}

impl GameMatch {
    pub fn new() -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let window_exists = capture(buffer.clone()).is_ok();

        GameMatch {
            timer: None,
            ingame_timer: None,
            restart_amount: 0,
            agents: None,
            current_image: buffer,
            window_exists,
            stage: Stage::Pick,
            match_result: Vec::with_capacity(2),
            prepare_next_stage: false,
            in_pause: false,
            count_restart: false,
            pause_tick_counter: 0,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        let task = match message {
            Message::Home => Action::Home,
            Message::ScanTick(_now) => {
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
                let image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();
                let shared_img = Arc::new(image);

                let prepare_next_stage = self.prepare_next_stage.clone();
                match self.stage {
                    Stage::Pick => Action::Run(Task::future(async move {
                        let run_timer = if prepare_next_stage {
                            let ocr = RunStage::get_timer_ocr(&*shared_img.clone());
                            Timer::from_raw_ocr(ocr.as_str())
                        } else {
                            None
                        };

                        if run_timer.is_some() {
                            return Message::ChangeStage(Stage::Run);
                        }

                        let agents = spawn_blocking(move || {
                            let ocr = PickStage::get_agent_ocr(&*shared_img.clone());
                            Agent::from_raw_ocr(&ocr)
                        })
                        .await
                        .unwrap();

                        Message::SetAgents(agents)
                    })),
                    Stage::Run => {
                        let img = shared_img.clone();
                        let ingame_timer_task = Task::future(async move {
                            let ingame_timer = spawn_blocking(move || {
                                let ocr = RunStage::get_timer_ocr(&img);
                                Timer::from_raw_ocr(ocr.as_str())
                            })
                            .await
                            .unwrap();

                            Message::SetIngameTimer(ingame_timer)
                        });

                        let img = shared_img.clone();
                        let res_timer_task = Task::future(async move {
                            let res_timer = spawn_blocking(move || {
                                let ocr = TimerStage::get_timer_ocr(&img);
                                Timer::from_raw_ocr(ocr.as_str())
                            })
                            .await
                            .unwrap();

                            match res_timer {
                                Some(t) => Message::SetTimer(t),
                                None => Message::None,
                            }
                        });

                        let img1 = shared_img.clone();
                        // let img2 = shared_img.clone();
                        let restart_task = Task::future(async move {
                            let (confirm, pause) = spawn_blocking(move || {
                                let ocr = PauseOcr::get_ocr(&img1);
                                let pause = Pause::from_raw_ocr(ocr);
                                let ocr = ConfirmOcr::get_ocr(&img1);
                                let confirm = ConfirmDialog::from_raw_ocr(&ocr);
                                (confirm, pause)
                            })
                            .await
                            .unwrap();

                            Message::SetRestart(pause.is_some(), confirm.is_some())
                        });

                        Action::Run(Task::batch(vec![
                            ingame_timer_task,
                            res_timer_task,
                            restart_task,
                        ]))
                    }
                    _ => Action::None,
                }
            }

            Message::SetRestart(is_paused, is_confirm) => {
                // println!(
                //     "SetRestart: is_paused: {}, is_confirm: {}",
                //     is_paused, is_confirm
                // );
                if !is_paused && !is_confirm && self.count_restart && !self.in_pause {
                    self.pause_tick_counter += 1;
                } else {
                    self.in_pause = is_paused;

                    if self.in_pause {
                        self.count_restart = false;
                    } else {
                        self.count_restart = is_confirm;
                    }

                    self.pause_tick_counter = 0;
                }

                // 3 seconds
                if self.pause_tick_counter >= 6 {
                    self.restart_amount += 1;
                    self.pause_tick_counter = 0;
                    self.in_pause = false;
                    self.count_restart = false;
                }

                Action::None
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
                self.timer = Some(timer);
                Action::Run(Task::done(Message::ChangeStage(Stage::Finished)))
            }
            Message::ChangeStage(stage) => {
                self.prepare_next_stage = false;
                self.stage = stage;

                match &self.stage {
                    Stage::Finished => {
                        let result = MatchResult {
                            agents: self.agents.take().expect("Expect agents is always Some"),
                            timer: self.timer.take().expect("expect timer to be Some"),
                            restart_amount: self.restart_amount,
                        };

                        self.match_result.push(result);

                        self.restart_amount = 0;
                        self.ingame_timer = None;

                        if self.match_result.len() == 2 {
                            self.stage = Stage::GameOver;
                        } else {
                            self.stage = Stage::Pick;
                        }
                    }
                    _ => {}
                }
                Action::None
            }
            _ => Action::None,
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

        let col_content = col_content.push(match self.stage {
            Stage::GameOver => {
                let mut iter = self.match_result.iter().enumerate();

                let mut cols = Vec::with_capacity(2);

                while let Some((idx, match_res)) = iter.next() {
                    let header = text(format!("Roster {}", idx + 1))
                        .size(20)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill);

                    let restarts =
                        text(format!("Restarts used: {}", match_res.restart_amount)).size(20);

                    let timer = text(match_res.timer.to_string())
                        .size(20)
                        .color(Color::WHITE)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill);

                    let agents = Self::agents(match_res.agents.as_slice());

                    cols.push(
                        column![header, column![restarts, timer, agents]]
                            .spacing(20)
                            .into(),
                    );
                }

                Column::from_vec(cols).width(Length::Fill).spacing(30)
            }
            _ => {
                let paused = text(format!("Paused: {}", &self.in_pause));
                let confirm = text(format!("Confirm opened: {}", &self.count_restart));
                let round = text(format!("Game {}", self.match_result.len() + 1));
                let restarts = text(format!("Restarts used: {}", self.restart_amount));
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

                column![paused, confirm, round, restarts, timer, agents].width(Length::Fill)
            }
        });

        let col_content = col_content.push(button("Home").on_press(Message::Home));

        col_content.spacing(20).into()
    }

    pub fn subscribtion(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(333)).map(Message::ScanTick)
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
