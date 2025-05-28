use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::widget::{text, Column, Row};
use iced::{time, Color, Element, Subscription, Task};
use image::RgbaImage;

mod capture;
use capture::{capture, get_characters};
use tokio::task::spawn_blocking;

mod agents;

fn main() {
    iced::application("hello world", App::update, App::view)
        .subscription(App::subscribtion)
        .window_size((500.0, 500.0))
        .run_with(|| {
            let app = App {
                content: String::new(),
                characters: None,
                current_image: Arc::new(Mutex::new(vec![])),
            };

            capture(app.current_image.clone());
            (app, Task::none())
        })
        .unwrap();
}

#[derive(Debug, Clone)]
enum Message {
    ScanTick(Instant),
    SetAgents(Vec<String>),
}

struct App {
    content: String,
    characters: Option<Vec<String>>,
    current_image: Arc<Mutex<Vec<u8>>>,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ScanTick(_now) => {
                println!("2 sec passed...");
                println!("Scanning");
                let image_buf = self.current_image.lock().unwrap().clone();
                let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();
                // let mut image = image.to_rgb8();

                // let timer = get_timer(&mut image);

                // if let Some((h, m, s)) = timer {
                //     self.content = format!("{}:{}:{}", h, m, s);
                // }
                return Task::perform(
                    async {
                        spawn_blocking(move || get_characters(&mut image))
                            .await
                            .unwrap()
                    },
                    Message::SetAgents,
                );
            }
            Message::SetAgents(agents) => {
                self.characters = Some(agents);
            }
            _ => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let col_content = Column::new();
        let timer_str = if self.content.len() == 0 {
            "No timer on the screen".to_string()
        } else {
            format!("{:#?}", self.content.as_str())
        };

        let timer = text(timer_str).size(20).color(Color::WHITE);

        let col_content = col_content.push(timer);

        let col_content = if self.characters.is_some() {
            let row = Row::from_iter(
                self.characters
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|c| text(c).size(20).color(Color::WHITE).into()),
            )
            .spacing(20);

            col_content.push(row)
        } else {
            col_content
        };

        col_content.spacing(20).into()
    }

    pub fn subscribtion(&self) -> Subscription<Message> {
        let tick = time::every(Duration::from_millis(500)).map(Message::ScanTick);
        tick
    }
}
