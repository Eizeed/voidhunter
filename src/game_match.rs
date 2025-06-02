use std::{
    ops::Sub,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use iced::{
    alignment::Horizontal,
    widget::{button, column, row, text, Column, Row},
    Color, Element, Length, Subscription, Task,
};
use image::RgbaImage;
use match_result::MatchResult;
use tokio::task::spawn_blocking;

use crate::{
    capture,
    ocr::{
        agents::{Agent, PickStage},
        challenge::{Challenge, ChallengeOcr},
        confirm::{ConfirmDialog, ConfirmOcr},
        frontier::{Frontier, FrontierOcr},
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
    SetFrontier(Option<Frontier>),
    SetAgents(Option<Vec<Option<Agent>>>),
    CheckChallenges(Option<Challenge>),
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
    Prepare,
    Run,
    Finished,
    GameOver,
}

pub struct GameMatch {
    frontier: Option<Frontier>,
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
    pub fn new() -> (Self, Task<Message>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let window_exists = capture(buffer.clone()).is_ok();

        (
            GameMatch {
                frontier: None,
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
            },
            Task::done(Message::ScanTick(Instant::now())),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        let task = match message {
            Message::Home => Action::Home,
            Message::ScanTick(now) => {
                // Ensure runtime process all messages as chain
                // and preventing consuming too much resources making
                // at least 333ms delays between messages.
                // It will guarantee that interval between messages
                // are Message execution time or 333ms depends what's
                // faster
                let elapsed = now.elapsed();
                if elapsed < Duration::from_millis(333) {
                    return Action::Run(Task::future(async move {
                        let diff = Duration::from_millis(333).sub(elapsed);
                        tokio::time::sleep(diff).await;
                        Message::ScanTick(now)
                    }));
                }

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
                    Stage::Pick => {
                        let img = shared_img.clone();
                        let frontier_task = Task::future(async move {
                            let frontier = spawn_blocking(move || {
                                let ocr = FrontierOcr::get_ocr(&img);
                                Frontier::from_raw_ocr(ocr)
                            })
                            .await
                            .unwrap();

                            Message::SetFrontier(frontier)
                        });

                        let img = shared_img.clone();
                        let agent_task = Task::future(async move {
                            let agents = spawn_blocking(move || {
                                let ocr = PickStage::get_agent_ocr(&img.clone());
                                Agent::from_raw_ocr(&ocr)
                            })
                            .await
                            .unwrap();

                            Message::SetAgents(agents)
                        });

                        let img = shared_img.clone();
                        let change_state_task = Task::future(async move {
                            if !prepare_next_stage {
                                return Message::None;
                            }

                            let challenges = spawn_blocking(move || {
                                let ocr = ChallengeOcr::get_ocr(&img);
                                Challenge::from_raw_ocr(ocr)
                            })
                            .await
                            .unwrap();

                            if challenges.is_some() {
                                Message::ChangeStage(Stage::Prepare)
                            } else {
                                Message::None
                            }
                        });

                        Action::Run(
                            frontier_task
                                .chain(agent_task)
                                .chain(change_state_task)
                                .chain(Task::done(Message::ScanTick(Instant::now()))),
                        )
                    }
                    Stage::Prepare => {
                        let task = Task::future(async move {
                            let ingame_timer = spawn_blocking(move || {
                                let ocr = RunStage::get_timer_ocr(&shared_img);
                                Timer::from_raw_ocr(ocr.as_str())
                            })
                            .await
                            .unwrap();

                            ingame_timer.map(|t| Message::SetIngameTimer(Some(t)))
                        })
                        .and_then(|_| Task::done(Message::ChangeStage(Stage::Run)));

                        let now = Instant::now();

                        Action::Run(task.chain(Task::done(Message::ScanTick(now))))
                    }
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

                            // println!("Confirm type: {:?}", confirm);

                            Message::SetRestart(pause.is_some(), confirm.is_some())
                        });

                        let now = Instant::now();

                        Action::Run(
                            Task::batch(vec![ingame_timer_task, res_timer_task, restart_task])
                                .chain(Task::done(Message::ScanTick(now))),
                        )
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
                    self.ingame_timer = None;
                    self.stage = Stage::Prepare;
                }

                Action::None
            }

            Message::SetFrontier(frontier) => {
                if let Some(frontier) = frontier {
                    self.prepare_next_stage = false;
                    self.frontier = Some(frontier);
                    self.agents = None;
                }

                Action::None
            }
            Message::SetAgents(agents) => {
                if self.agents.is_some() && agents.is_none() {
                    self.prepare_next_stage = true;
                } else {
                    if self.frontier.is_some() {
                        self.prepare_next_stage = false;
                        self.agents = agents;
                    }
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
                            agents: self.agents.take().expect("expect self.agents to be Some"),
                            timer: self.timer.take().expect("expect self.timer to be Some"),
                            restart_amount: self.restart_amount,
                            frontier: self
                                .frontier
                                .take()
                                .expect("expect self.frontier to be Some"),
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

                let mut total = 0;

                while let Some((idx, match_res)) = iter.next() {
                    let roster = text(format!("Roster {}", idx + 1))
                        .size(20)
                        .align_x(Horizontal::Center)
                        .width(Length::FillPortion(1));

                    let frontier = text(format!("Frontier: {:?}", match_res.frontier))
                        .size(20)
                        .align_x(Horizontal::Center)
                        .width(Length::FillPortion(1));

                    let header = row![roster, frontier];

                    let restarts =
                        text(format!("Restarts used: {}", match_res.restart_amount)).size(20);

                    let timer = text(match_res.timer.to_string())
                        .size(20)
                        .color(Color::WHITE)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill);

                    let agents = Self::agents(match_res.agents.as_slice());

                    total += match_res.timer.as_secs();

                    cols.push(
                        column![header, column![restarts, timer, agents]]
                            .spacing(20)
                            .into(),
                    );
                }

                let total = Timer::from(total);
                let total_timer = text(format!("Total timer: {}", total.to_string())).into();
                cols.push(total_timer);

                Column::from_vec(cols).width(Length::Fill).spacing(30)
            }
            _ => {
                let frontier_text = match &self.frontier {
                    Some(f) => format!("Selected frontier: {:?}", f),
                    None => format!("Frontier is not selected"),
                };
                let frontier = text(frontier_text);
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

                column![frontier, paused, confirm, round, restarts, timer, agents]
                    .width(Length::Fill)
            }
        });

        let col_content = col_content.push(button("Home").on_press(Message::Home));

        col_content.spacing(20).into()
    }

    pub fn subscribtion(&self) -> Subscription<Message> {
        // time::every(Duration::from_millis(400)).map(Message::ScanTick)
        Subscription::none()
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
