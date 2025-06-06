use image::{
    codecs::png::PngEncoder, ExtendedColorType, GenericImageView, ImageBuffer, ImageEncoder, Rgba, RgbaImage,
};
use tesseract::Tesseract;

#[derive(Debug, Clone)]
pub struct Hp;

impl Hp {
    pub fn from_image(image: &RgbaImage) -> Option<Self> {
        let ocr = HpOcr::get_ocr(image);
        Hp::from_raw_ocr(ocr)
    }
    
    pub fn from_raw_ocr(str: String) -> Option<Self> {
        let str = str.split('/').next()?;
        str.trim().parse::<u32>().ok()?;

        Some(Hp)
    }
}

pub struct HpOcr;

impl HpOcr {
    pub fn get_ocr(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        const X: u32 = 250;
        const Y: u32 = 85;
        const WIDTH: u32 = 90;
        const HEIGHT: u32 = 16;

        let hp_bar = image.view(X, Y, WIDTH, HEIGHT).to_image();
        hp_bar.save("hp.png").unwrap();

        let mut buffer = vec![];

        let png_encoder = PngEncoder::new(&mut buffer);
        png_encoder
            .write_image(hp_bar.as_raw(), WIDTH, HEIGHT, ExtendedColorType::Rgba8)
            .unwrap();

        let tesseract =
            Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

        let hp_bar = tesseract
            .set_image_from_mem(&buffer)
            .unwrap()
            .get_text()
            .unwrap()
            .trim()
            .to_string();

        hp_bar
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use image::RgbaImage;

    use crate::capture::capture_once;

    use super::*;

    #[test]
    fn hp_bar() {
        let buf = Arc::new(Mutex::new(vec![]));
        capture_once(buf.clone()).unwrap();

        let image_buf = buf.lock().unwrap().clone();
        let mut image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();

        let res = HpOcr::get_ocr(&mut image);
        let res = Hp::from_raw_ocr(res);

        println!("{res:#?}");
    }
}
