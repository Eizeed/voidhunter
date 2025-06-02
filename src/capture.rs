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
    once: bool,
}

pub struct Flags {
    pub buf: Arc<Mutex<Vec<u8>>>,
    pub once: bool,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = Flags;

    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let flags = ctx.flags;
        Ok(Capture {
            buf: flags.buf,
            once: flags.once,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        io::stdout().flush()?;
        let mut frame_buf = frame.buffer().unwrap();
        let buf = frame_buf.as_raw_buffer();

        let mut curr_buf = self.buf.lock().unwrap();
        curr_buf.clear();
        curr_buf.write(buf).unwrap();

        if self.once {
            capture_control.stop();
        }

        Ok(())
    }
}

pub fn capture(buf: Arc<Mutex<Vec<u8>>>) -> Result<(), CaptureError> {
    let window = Window::from_name("ZenlessZoneZero");

    let Ok(window) = window else {
        return Err(CaptureError::NotFound);
    };

    let settings = Settings::new(
        window,
        CursorCaptureSettings::Default,
        DrawBorderSettings::WithoutBorder,
        ColorFormat::Rgba8,
        Flags {
            buf: buf.clone(),
            once: false,
        },
    );

    Capture::start_free_threaded(settings).map_err(|_| CaptureError::NotFound)?;
    Ok(())
}

pub fn capture_once(buf: Arc<Mutex<Vec<u8>>>) -> Result<(), CaptureError> {
    let window = Window::from_name("ZenlessZoneZero");

    let Ok(window) = window else {
        return Err(CaptureError::NotFound);
    };

    let settings = Settings::new(
        window,
        CursorCaptureSettings::Default,
        DrawBorderSettings::WithoutBorder,
        ColorFormat::Rgba8,
        Flags {
            buf: buf.clone(),
            once: true,
        },
    );

    Capture::start(settings).map_err(|_| CaptureError::NotFound)?;
    Ok(())
}

#[derive(Debug)]
pub enum CaptureError {
    NotFound,
}

#[allow(dead_code)]
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
