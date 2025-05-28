use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use image::{
    codecs::png::PngEncoder, ExtendedColorType, GenericImage, ImageBuffer, ImageEncoder, Rgba,
};
use tesseract::Tesseract;
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
    window::Window,
};

pub struct Capture {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = Arc<Mutex<Vec<u8>>>;

    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let buf = ctx.flags;
        Ok(Capture { buf })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        io::stdout().flush()?;
        let mut frame_buf = frame.buffer().unwrap();
        let buf = frame_buf.as_raw_buffer();

        let mut curr_buf = self.buf.lock().unwrap();
        curr_buf.clear();
        curr_buf.write(buf).unwrap();

        Ok(())
    }
}

pub fn capture(buf: Arc<Mutex<Vec<u8>>>) {
    let window = Window::from_name("ZenlessZoneZero").unwrap();

    let settings = Settings::new(
        // Item to capture
        window,
        // Capture cursor settings
        CursorCaptureSettings::Default,
        // Draw border settings
        DrawBorderSettings::WithoutBorder,
        // The desired color format for the captured frame.
        ColorFormat::Rgba8,
        // Additional flags for the capture settings that will be passed to user defined `new` function.
        buf.clone(),
    );

    Capture::start_free_threaded(settings).expect("Screen capture failed");
}

pub fn get_timer(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) -> Option<(u32, u32, u32)> {
    const X_OFFSET: u32 = 450;
    const Y_OFFSET: u32 = 630;
    const WIDTH: u32 = 150;
    const HEIGHT: u32 = 33;

    let timer = image
        .sub_image(X_OFFSET, Y_OFFSET, WIDTH, HEIGHT)
        .to_image();

    let mut timer_png_bytes = Vec::new();

    let png_encoder = PngEncoder::new(&mut timer_png_bytes);
    png_encoder
        .write_image(timer.as_raw(), WIDTH, HEIGHT, ExtendedColorType::Rgba8)
        .unwrap();

    let tesseract =
        Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

    let timer_str = tesseract
        .set_image_from_mem(&timer_png_bytes)
        .unwrap()
        .get_text()
        .unwrap();

    let mut iter = timer_str.split(':');
    let h = iter.next()?.parse::<u32>().ok()?;
    let m = iter.next()?.parse::<u32>().ok()?;
    let s = iter.next()?.parse::<u32>().ok()?;

    Some((h, m, s))
}

pub fn get_characters(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) -> Vec<String> {
    const H1: u32 = 454;
    const H2: u32 = 902;

    const X1: u32 = 372;
    const X2: u32 = 846;
    const X3: u32 = 1321;

    const DIFF: u32 = 132;

    const WIDTH: u32 = 190;
    const HEIGHT: u32 = 33;

    let char_pos = vec![
        (X1, H1),
        (X2, H1),
        (X3, H1),
        (X1 - DIFF, H2),
        (X2 - DIFF, H2),
        (X3 - DIFF, H2),
    ];

    let mut agent_names = Vec::new();
    let mut buffer = Vec::new();

    for (x, y) in char_pos.into_iter() {
        let agent_image = image.sub_image(x, y, WIDTH, HEIGHT).to_image();
        agent_image.save(format!("char-{}.png", x)).unwrap();

        let png_encoder = PngEncoder::new(&mut buffer);
        png_encoder
            .write_image(
                agent_image.as_raw(),
                WIDTH,
                HEIGHT,
                ExtendedColorType::Rgba8,
            )
            .unwrap();

        let tesseract =
            Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

        let agent = tesseract
            .set_image_from_mem(&buffer)
            .unwrap()
            .get_text()
            .unwrap()
            .trim()
            .to_string();

        agent_names.push(agent);
        buffer.clear();
    }

    agent_names
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn chars() {
        // let image_buf = capture();
        // let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();
        // let agents = get_characters(&mut image);
        // println!("{agents:#?}");
    }

    #[test]
    fn f() {
        let output = Command::new("tesseract")
            .arg("scn.png")
            .arg("stdout")
            .output()
            .unwrap();

        let output = String::from_utf8(output.stdout).unwrap();
        let mut split = output.trim().split(":");
        let hours: u32 = split.next().unwrap().parse().unwrap();
        let minutes: u32 = split.next().unwrap().parse().unwrap();
        let seconds: u32 = split.next().unwrap().parse().unwrap();
        println!("hours: {}", hours);
        println!("minutes: {}", minutes);
        println!("seconds: {}", seconds);
    }
}
