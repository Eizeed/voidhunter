use image::{
    codecs::png::PngEncoder, ExtendedColorType, GenericImageView, ImageBuffer, ImageEncoder, Rgba,
};
use tesseract::Tesseract;

#[derive(Debug, Clone)]
pub struct Pause;

impl Pause {
    pub fn from_raw_ocr((restart, exit): (String, String)) -> Option<Self> {
        let mut iter = restart.split_whitespace();
        let restart_res;

        let w = iter.next();
        if let Some(w) = w {
            restart_res = w == "Restart";
        } else {
            restart_res = false;
        }

        let mut iter = exit.split_whitespace();
        let exit_res;

        let w = iter.next();
        if let Some(w) = w {
            exit_res = w == "Exit";
        } else {
            exit_res = false;
        }

        if restart_res || exit_res {
            Some(Pause)
        } else {
            None
        }
    }
}

pub struct PauseOcr;

impl PauseOcr {
    pub fn get_ocr(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> (String, String) {
        const X1: u32 = 1290;
        const X2: u32 = 1620;
        const Y: u32 = 1007;
        const WIDTH: u32 = 210;
        const HEIGHT: u32 = 45;

        let restart = image.view(X1, Y, WIDTH, HEIGHT).to_image();
        // let restart = &contrast(&grayscale(&restart), 100.0);
        restart.save("pause_r.png");

        let pause = image.view(X2, Y, WIDTH, HEIGHT).to_image();
        // let pause = &contrast(&grayscale(&pause), 100.0);
        pause.save("pause_p.png");

        let mut buffer_restart = vec![];
        let png_encoder = PngEncoder::new(&mut buffer_restart);
        png_encoder
            .write_image(restart.as_raw(), WIDTH, HEIGHT, ExtendedColorType::Rgba8)
            .unwrap();

        let mut buffer_exit = vec![];
        let png_encoder = PngEncoder::new(&mut buffer_exit);
        png_encoder
            .write_image(pause.as_raw(), WIDTH, HEIGHT, ExtendedColorType::Rgba8)
            .unwrap();

        let tesseract =
            Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

        let restart = tesseract
            .set_image_from_mem(&buffer_restart)
            .unwrap()
            .get_text()
            .unwrap()
            .trim()
            .to_string();

        let tesseract =
            Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

        let exit = tesseract
            .set_image_from_mem(&buffer_exit)
            .unwrap()
            .get_text()
            .unwrap()
            .trim()
            .to_string();

        (restart, exit)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use image::RgbaImage;

    use crate::capture::capture_once;

    use super::*;

    #[test]
    fn pause() {
        let buf = Arc::new(Mutex::new(vec![]));
        capture_once(buf.clone()).unwrap();

        let image_buf = buf.lock().unwrap().clone();
        let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();

        let res = PauseOcr::get_ocr(&mut image);
        let res = Pause::from_raw_ocr(res);

        println!("{res:#?}");
    }
}
