use std::time::{Duration, Instant};

use iced::widget::{text, Column, Row};
use iced::{time, Color, Element, Subscription, Task};

mod capture;
use capture::{capture, get_characters, get_timer};
use image::RgbaImage;

fn main() {
    iced::application("hello world", App::update, App::view)
        .subscription(App::subscribtion)
        .window_size((500.0, 500.0))
        .run_with(|| {
            (
                App {
                    content: String::new(),
                    characters: None,
                },
                Task::none(),
            )
        })
        .unwrap();
}

#[derive(Debug, Clone)]
enum Message {
    ScanTick(Instant),
}

struct App {
    content: String,
    characters: Option<Vec<String>>,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ScanTick(_now) => {
                println!("2 sec passed...");
                println!("Scanning");
                let image_buf = capture();
                let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();

                let timer = get_timer(&mut image);
                let characters = get_characters(&mut image);

                if let Some((h, m, s)) = timer {
                    self.content = format!("{}:{}:{}", h, m, s);
                }
                self.characters = Some(characters);
            }
        };
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let col_content = Column::new();
        let timer_str = if self.content.len() == 0 {
            "No timer on the screen"
        } else {
            self.content.as_str()
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
        let tick = time::every(Duration::from_secs(2)).map(Message::ScanTick);
        tick
    }
}
