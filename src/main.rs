use image::codecs::png::PngEncoder;
use std::time::{Duration, Instant};
use tesseract::Tesseract;

use iced::widget::text;
use iced::{time, Color, Element, Subscription, Task};

mod capture;
use capture::capture;
use image::{ExtendedColorType, GenericImage, ImageEncoder, RgbaImage};

fn main() {
    iced::application("hello world", App::update, App::view)
        .subscription(App::subscribtion)
        .window_size((500.0, 500.0))
        .run_with(|| {
            (
                App {
                    content: String::new(),
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
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ScanTick(_now) => {
                println!("2 sec passed...");
                println!("Scanning");
                let image_buf = capture();
                let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();
                // let mut image = image.to_luma8();

                let timer = image.sub_image(450, 630, 150, 33).to_image();

                // -----------------------------------
                //           Only for debug
                let path = format!("timer.png");
                timer.save(path).unwrap();
                // -----------------------------------

                let mut timer_png_bytes = vec![];
                let png_encoder = PngEncoder::new(&mut timer_png_bytes);
                png_encoder
                    .write_image(timer.as_raw(), 150, 33, ExtendedColorType::Rgba8)
                    .unwrap();

                let content =
                    Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng"))
                        .unwrap();
                let mut content = content.set_image_from_mem(&timer_png_bytes).unwrap();
                let content = content.get_text();
                self.content = content.unwrap();
            }
        };
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        text(&self.content).size(20).color(Color::WHITE).into()
    }

    pub fn subscribtion(&self) -> Subscription<Message> {
        let tick = time::every(Duration::from_secs(2)).map(Message::ScanTick);
        tick
    }
}
