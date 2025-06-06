use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

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

#[cfg(test)]
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
