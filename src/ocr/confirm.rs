use image::{
    codecs::png::PngEncoder,
    imageops::{contrast, grayscale},
    ExtendedColorType, GenericImageView, ImageBuffer, ImageEncoder, Rgba,
};
use imageproc::{distance_transform::Norm, morphology::erode};
use tesseract::Tesseract;

#[derive(Debug, Clone)]
pub enum ConfirmDialog {
    Opaque,
    Restart,
    Exit,
}

impl ConfirmDialog {
    pub fn from_raw_ocr(message: &str) -> Option<Self> {
        if message.contains("Leave") {
            return Some(ConfirmDialog::Exit);
        }
        if message.contains("Restart") {
            return Some(ConfirmDialog::Restart);
        }

        if message.contains("battle") {
            return Some(ConfirmDialog::Opaque);
        }

        return None;
    }
}

pub struct ConfirmOcr;

impl ConfirmOcr {
    pub fn get_ocr(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        const X: u32 = 784;
        const Y: u32 = 510;
        const WIDTH: u32 = 351;
        const HEIGHT: u32 = 29;

        let restart = image.view(X, Y, WIDTH, HEIGHT).to_image();
        let restart = &contrast(&grayscale(&restart), 100.0);
        let restart = erode(&restart, Norm::LInf, 2);

        let mut buffer = vec![];
        let png_encoder = PngEncoder::new(&mut buffer);
        png_encoder
            .write_image(restart.as_raw(), WIDTH, HEIGHT, ExtendedColorType::L8)
            .unwrap();

        let tesseract =
            Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

        let restart = tesseract
            .set_image_from_mem(&buffer)
            .unwrap()
            .get_text()
            .unwrap()
            .trim()
            .to_string();

        restart
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use image::RgbaImage;

    use crate::capture::capture_once;

    use super::*;

    #[test]
    fn restart() {
        let buf = Arc::new(Mutex::new(vec![]));
        capture_once(buf.clone()).unwrap();

        let image_buf = buf.lock().unwrap().clone();
        let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();

        let res = ConfirmOcr::get_ocr(&mut image);
        let res = ConfirmDialog::from_raw_ocr(&res);
        println!("{res:#?}");
    }
}
