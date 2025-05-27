use std::{
    io::{self, Write},
    sync::mpsc::Sender,
    time::Duration,
};

use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    encoder,
    frame::{Frame, ImageFormat},
    graphics_capture_api::InternalCaptureControl,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
    window::Window,
};

pub struct Capture {
    buffer: Vec<u8>,
    encoder: encoder::ImageEncoder,
    rx: Sender<Vec<u8>>,
}

impl GraphicsCaptureApiHandler for Capture {
    // The type of flags used to get the values from the settings.
    type Flags = Sender<Vec<u8>>;

    // The type of error that can be returned from `CaptureControl` and `start` functions.
    type Error = Box<dyn std::error::Error + Send + Sync>;

    // Function that will be called to create a new instance. The flags can be passed from settings.
    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let encoder = encoder::ImageEncoder::new(ImageFormat::Png, ColorFormat::Rgba8);
        let rx = ctx.flags;
        Ok(Capture {
            buffer: vec![],
            encoder,
            rx,
        })
    }

    // Called every time a new frame is available.
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        io::stdout().flush()?;

        // let start_w = 450;
        // let start_h = 630;
        // let end_w = 600;
        // let end_h = 666;

        // let mut frame = frame.buffer_crop(start_w, start_h, end_w, end_h).unwrap();
        // frame.save_as_image("scn.png", ImageFormat::Png).unwrap();
        self.rx
            .send(frame.buffer().unwrap().as_raw_buffer().to_vec())
            .unwrap();
        // let img = RgbImage::from_vec(
        //     end_w - start_w,
        //     end_h - start_h,
        //     frame.as_raw_buffer().to_vec(),
        // )
        // .unwrap();
        // let w = frame.width();
        // let h = frame.height();
        // self.encoder.as_mut().unwrap().encode(&frame.save_as_image(path, format), w, h);

        // Note: The frame has other uses too, for example, you can save a single frame to a file, like this:
        // frame.save_as_image("frame.png", ImageFormat::Png)?;
        // Or get the raw data like this so you have full control:
        // let data = frame.buffer()?;
        capture_control.stop();

        Ok(())
    }

    // Optional handler called when the capture item (usually a window) closes.
    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture session ended");

        Ok(())
    }
}

pub fn capture() -> Vec<u8> {
    // Gets the foreground window, refer to the docs for other capture items
    // let primary_monitor = Monitor::primary().expect("There is no primary monitor");
    let window = Window::from_name("ZenlessZoneZero").unwrap();

    let (rx, tx) = std::sync::mpsc::channel();

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
        rx,
    );

    // Starts the capture and takes control of the current thread.
    // The errors from handler trait will end up here
    Capture::start_free_threaded(settings).expect("Screen capture failed");
    let res = tx.recv_timeout(Duration::from_secs(1));
    match res {
        Ok(buf) => buf,
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use std::{process::Command, time::Duration};

    use image::RgbaImage;
    use windows_capture::{
        capture::GraphicsCaptureApiHandler, settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings}, window::Window
    };

    use super::Capture;

    #[test]
    fn cap() {
        // Gets the foreground window, refer to the docs for other capture items
        // let primary_monitor = Monitor::primary().expect("There is no primary monitor");
        let window = Window::from_name("ZenlessZoneZero").unwrap();

        let (rx, tx) = std::sync::mpsc::channel();

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
            rx,
        );

        // Starts the capture and takes control of the current thread.
        // The errors from handler trait will end up here
        Capture::start(settings).expect("Screen capture failed");
        let res = tx.recv_timeout(Duration::from_secs(2));
        match res {
            Ok(buf) => {
                let img = RgbaImage::from_vec(1920, 1080, buf).unwrap();
                img.save("Atest.png").unwrap();
            },
            Err(_) => {}
        };
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
