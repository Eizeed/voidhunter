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

use crate::{
    bitmap::BitmapU16,
    capture,
    ocr::{
        agents::Agent, challenge::Challenge, confirm::ConfirmDialog, frontier::Frontier, hp::Hp,
        is_black_screen, loading::Loading, pause::Pause, timer::Timer,
    },
    spawn_blocking,
};

mod match_result;
mod transition;

pub enum Action {
    Run(Task<Message>),
    Home,
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    Home,

    ScanTick(Instant),
    SetFrontier(Option<Frontier>),
    SetAgents(Option<Vec<Option<Agent>>>),
    SetChallenges(Option<Challenge>),
    SetHp(Option<Hp>),
    SetIngameTimer(Option<Timer>),
    SetTimer(Option<Timer>),
    SetLoading(Option<Loading>),
    SetPause(Option<Pause>),
    SetConfirmDialog(Option<ConfirmDialog>),
    SetBlackscreen(bool),

    CheckState,

    ChangeStage(Stage),
    SetRestart(bool, bool),
    SetVisibleHp(bool),
    // SetPause(bool),
    // SetConfirm(bool),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    Pick,
    FirstHalf(HalfStage),
    SecondHalf(HalfStage),
    Finished,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HalfStage {
    Prepare,
    Run,
    Cleared,
}

#[derive(Debug, Clone)]
pub struct GameMatch {
    window_exists: bool,
    current_image: Arc<Mutex<Vec<u8>>>,
    match_results: Vec<MatchResult>,

    game: GameState,

    player_state: PlayerAction,
}

#[derive(Debug, Clone)]
pub struct GameState {
    frontier: Option<Frontier>,
    agents: Option<Vec<Option<Agent>>>,
    ingame_timer: Option<Timer>,
    res_timer: Option<Timer>,
    restart_amount: u8,
    is_dirty: bool,
    stage: Stage,
    visibility_flags: BitmapU16,
    tick: u32,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            frontier: None,
            agents: None,
            ingame_timer: None,
            res_timer: None,
            restart_amount: 0,
            is_dirty: false,
            stage: Stage::Pick,
            visibility_flags: 0.into(),
            tick: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerAction {
    None,
    Pause,
    RestartDialog,
    ExitDialog,
}

impl GameMatch {
    pub fn new() -> (Self, Task<Message>) {
        // 3 MB dedicated for image capturing to avoid additional allocations
        // NOTE: the biggest image i've seen is 2297 KB.
        let buffer = Arc::new(Mutex::new(Vec::with_capacity(1024 * 1024 * 3)));
        let window_exists = capture(buffer.clone()).is_ok();

        (
            GameMatch {
                window_exists,
                current_image: buffer,
                match_results: Vec::with_capacity(2),
                game: GameState::new(),
                player_state: PlayerAction::None,
            },
            Task::done(Message::ScanTick(Instant::now())),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        let task = match message {
            Message::Home => Action::Home,

            Message::ScanTick(now) => {
                const DIFF: u64 = 250;
                // Need to make at least delay.
                // We have to chain tasks recursivly because
                // update function doesn't wait for the prev task
                // to be completed unless chained.
                // It means that Message::ScanTick will trigget
                // every time and it doesn't guarantee that prev
                // ScanTick was completed. It can lead to data races.
                let elapsed = now.elapsed();
                if elapsed < Duration::from_millis(DIFF) {
                    return Action::Run(Task::future(async move {
                        let diff = Duration::from_millis(DIFF).sub(elapsed);
                        tokio::time::sleep(diff).await;
                        Message::ScanTick(now)
                    }));
                }

                if !self.window_exists {
                    let res = capture(self.current_image.clone());
                    if res.is_ok() {
                        self.window_exists = true;
                    }

                    return Action::Run(Task::done(Message::ScanTick(Instant::now())));
                }

                let image_buf = self.current_image.lock().unwrap().clone();
                if image_buf.is_empty() {
                    return Action::Run(Task::done(Message::ScanTick(Instant::now())));
                }
                let image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();
                let shared_img = Arc::new(image);

                let task = match self.game.stage {
                    Stage::Pick => {
                        let img = Arc::clone(&shared_img);
                        let frontier_task = Task::future(async move {
                            let frontier = spawn_blocking!(Frontier::from_image(&img));
                            Message::SetFrontier(frontier)
                        });

                        let img = Arc::clone(&shared_img);
                        let agents_task = Task::future(async move {
                            let agents = spawn_blocking!(Agent::from_image(&img));
                            Message::SetAgents(agents)
                        });

                        let img = Arc::clone(&shared_img);
                        let challenges_task = Task::future(async move {
                            let challenges = spawn_blocking!(Challenge::from_image(&img));
                            Message::SetChallenges(challenges)
                        });

                        let img = Arc::clone(&shared_img);
                        let hp_task = Task::future(async move {
                            let challenges = spawn_blocking!(Hp::from_image(&img));
                            Message::SetHp(challenges)
                        });

                        frontier_task
                            .chain(agents_task)
                            .chain(challenges_task)
                            .chain(hp_task)
                    }
                    Stage::FirstHalf(ref half_stage) => {
                        let img = Arc::clone(&shared_img);
                        let ingame_timer_task = Task::future(async move {
                            let ingame_timer = Timer::ingame_from_image(&img);
                            Message::SetIngameTimer(ingame_timer)
                        });

                        let img = Arc::clone(&shared_img);
                        let hp_task = Task::future(async move {
                            let challenges = spawn_blocking!(Hp::from_image(&img));
                            Message::SetHp(challenges)
                        });

                        let img = Arc::clone(&shared_img);
                        let loading_task = Task::future(async move {
                            let loading = spawn_blocking!(Loading::from_image(&img));
                            Message::SetLoading(loading)
                        });

                        let img = Arc::clone(&shared_img);
                        let pause_task = Task::future(async move {
                            let pause = spawn_blocking!(Pause::from_image(&img));
                            Message::SetPause(pause)
                        });

                        let img = Arc::clone(&shared_img);
                        let confirm_task = Task::future(async move {
                            let confirm_dialog = spawn_blocking!(ConfirmDialog::from_image(&img));
                            Message::SetConfirmDialog(confirm_dialog)
                        });

                        let black_screen = match half_stage {
                            HalfStage::Run => {
                                let img = Arc::clone(&shared_img);
                                let all_black = is_black_screen(&img);
                                // println!("Scanning. Is black_screen: {all_black}");
                                Task::done(Message::SetBlackscreen(all_black))
                            }
                            _ => Task::none(),
                        };

                        Task::batch(vec![
                            ingame_timer_task,
                            hp_task,
                            pause_task,
                            confirm_task,
                            loading_task,
                            black_screen,
                        ])
                    }
                    Stage::SecondHalf(_) => {
                        let img = Arc::clone(&shared_img);
                        let ingame_timer_task = Task::future(async move {
                            let ingame_timer = Timer::ingame_from_image(&img);
                            Message::SetIngameTimer(ingame_timer)
                        });

                        let img = Arc::clone(&shared_img);
                        let hp_task = Task::future(async move {
                            let challenges = spawn_blocking!(Hp::from_image(&img));
                            Message::SetHp(challenges)
                        });

                        let img = Arc::clone(&shared_img);
                        let loading_task = Task::future(async move {
                            let loading = spawn_blocking!(Loading::from_image(&img));
                            Message::SetLoading(loading)
                        });

                        let img = Arc::clone(&shared_img);
                        let pause_task = Task::future(async move {
                            let pause = spawn_blocking!(Pause::from_image(&img));
                            Message::SetPause(pause)
                        });

                        let img = Arc::clone(&shared_img);
                        let confirm_task = Task::future(async move {
                            let confirm_dialog = spawn_blocking!(ConfirmDialog::from_image(&img));
                            Message::SetConfirmDialog(confirm_dialog)
                        });

                        let res_timer_task =
                            if self.game.stage != Stage::SecondHalf(HalfStage::Prepare) {
                                let img = Arc::clone(&shared_img);
                                Task::future(async move {
                                    let res_timer = spawn_blocking!(Timer::res_from_image(&img));
                                    Message::SetTimer(res_timer)
                                })
                            } else {
                                Task::none()
                            };

                        Task::batch(vec![
                            ingame_timer_task,
                            hp_task,
                            res_timer_task,
                            pause_task,
                            confirm_task,
                            loading_task,
                        ])
                    }
                    Stage::Finished => Task::none(),
                    _ => Task::none(),
                };

                let now = Instant::now();

                if self.game.stage == Stage::SecondHalf(HalfStage::Cleared) {
                    println!("lollll");
                }

                Action::Run(
                    task.chain(Task::done(Message::CheckState))
                        .chain(Task::done(Message::ScanTick(now))),
                )
            }

            Message::SetFrontier(frontier) => {
                self.game.visibility_flags.set_frontier(frontier.is_some());
                frontier.map(|f| self.game.frontier = Some(f));

                Action::None
            }
            Message::SetAgents(agents) => {
                self.game.visibility_flags.set_agents(agents.is_some());
                agents.map(|a| self.game.agents = Some(a));

                Action::None
            }
            Message::SetChallenges(challenges) => {
                self.game
                    .visibility_flags
                    .set_challenges(challenges.is_some());

                Action::None
            }
            Message::SetHp(hp) => {
                self.game.visibility_flags.set_hp(hp.is_some());

                Action::None
            }
            Message::SetIngameTimer(ingame_timer) => {
                self.game
                    .visibility_flags
                    .set_ingame_timer(ingame_timer.is_some());
                ingame_timer.map(|t| self.game.ingame_timer = Some(t));

                Action::None
            }
            Message::SetTimer(res_timer) => {
                self.game
                    .visibility_flags
                    .set_res_timer(res_timer.is_some());
                res_timer.map(|t| self.game.res_timer = Some(t));

                Action::None
            }
            Message::SetLoading(loading) => {
                self.game.visibility_flags.set_loading(loading.is_some());
                Action::None
            }
            Message::SetPause(pause) => {
                self.game.visibility_flags.set_pause(pause.is_some());
                if let Some(_) = pause {
                    self.player_state = PlayerAction::Pause;
                };

                Action::None
            }
            Message::SetConfirmDialog(confirm) => {
                self.game
                    .visibility_flags
                    .set_confirm_dialog(confirm.is_some());

                if let Some(confirm) = confirm {
                    match confirm {
                        ConfirmDialog::Restart => self.player_state = PlayerAction::RestartDialog,
                        ConfirmDialog::Exit => self.player_state = PlayerAction::ExitDialog,
                        _ => {}
                    }
                };

                Action::None
            }
            Message::SetBlackscreen(b) => {
                self.game.visibility_flags.set_blackscreen(b);
                Action::None
            }

            Message::CheckState => {
                if !self.game.visibility_flags.pause()
                    && !self.game.visibility_flags.confirm_dialog()
                    && (self.game.visibility_flags.hp()
                        || self.game.visibility_flags.ingame_timer()
                        || self.game.visibility_flags.res_timer())
                {
                    self.player_state = PlayerAction::None;
                }

                let transition = self
                    .transition()
                    .map(|s| Message::ChangeStage(s))
                    .unwrap_or(Message::None);

                Action::Run(Task::done(transition))
            }

            Message::ChangeStage(stage) => {
                self.game.stage = stage;
                self.player_state = PlayerAction::None;

                println!("Setting stage. Current stage: {:#?}", self.game.stage);
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
        let current_stage = text(format!("{:?}", self.game.stage))
            .size(25)
            .color(Color::WHITE);

        let col_content = col_content.push(current_stage);

        let col_content = col_content.push(match self.game.stage {
            Stage::GameOver => {
                let mut iter = self.match_results.iter().enumerate();

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
                let frontier_text = match &self.game.frontier {
                    Some(f) => format!("Selected frontier: {:?}", f),
                    None => format!("Frontier is not selected"),
                };
                let frontier = text(frontier_text);
                let paused = text(format!(
                    "Paused: {}",
                    matches!(&self.player_state, PlayerAction::Pause)
                ));
                let confirm = text(format!(
                    "Confirm opened: {}",
                    matches!(&self.player_state, PlayerAction::RestartDialog)
                ));
                let round = text(format!("Game {}", self.match_results.len() + 1));
                let ticks = text(format!("Ticks {}", self.game.tick));
                let restarts = text(format!("Restarts used: {}", self.game.restart_amount));
                let player_action = text(format!("Player Action: {:?}", self.player_state));
                let hp_visible = text(format!("Hp visible: {}", self.game.visibility_flags.hp()));
                let ingame_timer = text(format!(
                    "ingame timer visible: {}",
                    self.game.visibility_flags.ingame_timer()
                ));
                let res_timer_visible = text(format!(
                    "res timer visible: {}",
                    self.game.visibility_flags.res_timer()
                ));
                let res_timer = text(format!("res timer: {:?}", self.game.res_timer));
                let timer = if let Some(timer) = &self.game.ingame_timer {
                    text(format!("Ingame timer: {}", timer.to_string()))
                        .size(20)
                        .color(Color::WHITE)
                } else {
                    text("No timer on the screen").size(20).color(Color::WHITE)
                };

                let agents: Element<_, _, _> = match self.game.agents.as_ref() {
                    Some(agents) => {
                        let header = text("Chosen agents:").size(20).color(Color::WHITE);
                        let agents = Self::agents(agents.as_slice());
                        column![header, agents].into()
                    }
                    None => text("Not in Pick Stage").into(),
                };

                column![
                    frontier,
                    paused,
                    confirm,
                    ticks,
                    round,
                    restarts,
                    player_action,
                    hp_visible,
                    ingame_timer,
                    res_timer_visible,
                    res_timer,
                    timer,
                    agents
                ]
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
