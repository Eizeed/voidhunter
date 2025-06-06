use image::{
    codecs::png::PngEncoder, ExtendedColorType, GenericImageView, ImageBuffer, ImageEncoder, Rgba, RgbaImage,
};
use tesseract::Tesseract;

#[derive(Debug, Clone)]
pub enum Frontier {
    Fifth,
    Sixth,
    Seventh,
    NotPickable,
}

impl Frontier {
    pub fn from_image(image: &RgbaImage) -> Option<Self> {
        let ocr = FrontierOcr::get_ocr(image);
        Frontier::from_raw_ocr(ocr)
    }
    
    pub fn from_raw_ocr(frontier: String) -> Option<Self> {
        let mut iter = frontier.split_whitespace();
        let num = iter.next()?;
        let postfix = iter.next()?;

        if postfix != "Frontier" {
            return None;
        }

        match num {
            "First" | "Second" | "Third" | "Fourth" => Some(Frontier::NotPickable),
            "Fifth" => Some(Frontier::Fifth),
            "Sixth" => Some(Frontier::Sixth),
            "Seventh" => Some(Frontier::Seventh),
            _ => None,
        }
    }
}

pub struct FrontierOcr;

impl FrontierOcr {
    pub fn get_ocr(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        const X: u32 = 366;
        const Y: u32 = 229;
        const WIDTH: u32 = 289;
        const HEIGHT: u32 = 28;

        let frontier_title = image.view(X, Y, WIDTH, HEIGHT).to_image();
        // frontier_title.save("front.png").unwrap();

        let mut buffer = vec![];

        let png_encoder = PngEncoder::new(&mut buffer);
        png_encoder
            .write_image(
                frontier_title.as_raw(),
                WIDTH,
                HEIGHT,
                ExtendedColorType::Rgba8,
            )
            .unwrap();

        let tesseract =
            Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

        let frontier = tesseract
            .set_image_from_mem(&buffer)
            .unwrap()
            .get_text()
            .unwrap()
            .trim()
            .to_string();

        frontier
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use image::RgbaImage;

    use crate::capture::capture_once;

    use super::*;

    #[test]
    fn frontier() {
        let buf = Arc::new(Mutex::new(vec![]));
        capture_once(buf.clone()).unwrap();

        let image_buf = buf.lock().unwrap().clone();
        let image = RgbaImage::from_vec(1920, 1080, image_buf).unwrap();

        let res = FrontierOcr::get_ocr(&image);
        let res = Frontier::from_raw_ocr(res);
        println!("{res:#?}");
    }
}
