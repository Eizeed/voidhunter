use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use agents::{Agent, PickStage};
use iced::alignment::Horizontal;
use iced::widget::{text, Column, Row};
use iced::{time, Color, Element, Length, Subscription, Task};
use image::RgbaImage;

mod capture;
use capture::capture;
use timer::{RunStage, Timer, TimerStage};
use tokio::task::spawn_blocking;

mod agents;
mod timer;

fn main() {
    iced::application("hello world", App::update, App::view)
        .subscription(App::subscribtion)
        .window_size((500.0, 500.0))
        .run_with(|| {
            let buffer = Arc::new(Mutex::new(vec![]));
            let window_exists = capture(buffer.clone()).is_ok();

            let app = App {
                timer: String::new(),
                agents: None,
                current_image: buffer,
                window_exists,
                stage: Stage::Pick,
                check_next_stage: false,
            };

            (app, Task::none())
        })
        .unwrap();
}

#[derive(Debug, Clone)]
enum Message {
    ScanTick(Instant),
    SetAgents(Option<Vec<Option<Agent>>>),
    SetTimer(Option<Timer>),
    ChangeStage(Stage),
}

struct App {
    timer: String,
    agents: Option<Vec<Option<Agent>>>,
    current_image: Arc<Mutex<Vec<u8>>>,
    window_exists: bool,
    stage: Stage,
    check_next_stage: bool,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let task = match message {
            Message::ScanTick(_now) => {
                println!("500 ms passed...");
                println!("Scanning");
                println!("");

                if !self.window_exists {
                    let res = capture(self.current_image.clone());
                    if res.is_ok() {
                        self.window_exists = true;
                    }

                    return Task::none();
                }

                let image_buf = self.current_image.lock().unwrap().clone();
                if image_buf.is_empty() {
                    return Task::none();
                }

                let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();
                // let mut image = image.to_rgb8();

                // let timer = get_timer(&mut image);

                // if let Some((h, m, s)) = timer {
                //     self.content = format!("{}:{}:{}", h, m, s);
                // }
                let check_next_stage = self.check_next_stage.clone();
                match self.stage {
                    Stage::Pick => Task::future(async move {
                        let run_timer = if check_next_stage {
                            let ocr = RunStage::get_timer_ocr(&mut image);
                            Timer::from_raw_ocr(ocr.as_str())
                        } else {
                            None
                        };

                        if run_timer.is_some() {
                            return Message::ChangeStage(Stage::Timer);
                        }

                        let agents = spawn_blocking(move || {
                            let ocr = PickStage::get_agent_ocr(&mut image);
                            Agent::from_raw_ocr(&ocr)
                        })
                        .await
                        .unwrap();

                        Message::SetAgents(agents)
                    }),
                    Stage::Timer => Task::future(async {
                        let timer = spawn_blocking(move || {
                            let ocr = TimerStage::get_timer_ocr(&mut image);
                            Timer::from_raw_ocr(ocr.as_str())
                        })
                        .await
                        .unwrap();

                        Message::SetTimer(timer)
                    }),
                    _ => Task::none(),
                }
            }
            Message::SetAgents(agents) => {
                if self.agents.is_some() && agents.is_none() {
                    self.check_next_stage = true;
                } else {
                    self.check_next_stage = false;
                    self.agents = agents;
                }
                Task::none()
            }
            Message::SetTimer(timer) => {
                if self.timer.len() > 0 && timer.is_none() {
                    return Task::done(Message::ChangeStage(Stage::Finished));
                } else {
                    self.timer = timer.map(|t| t.to_string()).unwrap_or_default();
                }
                Task::none()
            }
            Message::ChangeStage(stage) => {
                self.check_next_stage = false;
                self.stage = stage;
                Task::none()
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

        let change_stage = text(format!("Change state: {:?}", self.check_next_stage))
            .size(20)
            .color(Color::WHITE);

        let col_content = col_content.push(change_stage);

        let timer_str = if self.timer.len() == 0 {
            "No timer on the screen".to_string()
        } else {
            match self.stage {
                Stage::Timer => {
                    format!("{}", self.timer.as_str())
                },
                Stage::Finished => {
                    format!("Your timer is {}", self.timer.as_str())
                },
                _ => unreachable!("Timer must be set if this branch is hit"),
            }
        };

        let timer = text(timer_str).size(20).color(Color::WHITE);

        let col_content = col_content.push(timer);

        let col_content = if self.agents.is_some() {
            let (first_team, second_team) = self.agents.as_ref().unwrap().as_slice().split_at(3);

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

            let col_content = col_content.push(row_1);
            col_content.push(row_2)
        } else {
            col_content.push(text("Not in Pick Stage"))
        };

        col_content.spacing(20).into()
    }

    pub fn subscribtion(&self) -> Subscription<Message> {
        let tick = time::every(Duration::from_millis(500)).map(Message::ScanTick);
        tick
    }
}

#[derive(Debug, Clone)]
pub enum Stage {
    Pick,
    Run,
    Timer,
    Finished,
}
