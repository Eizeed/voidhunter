use image::{
    codecs::png::PngEncoder, ExtendedColorType, GenericImageView, ImageEncoder, RgbaImage,
};
use tesseract::Tesseract;

#[derive(Debug, Clone)]
pub struct Loading;

impl Loading {
    pub fn from_image(image: &RgbaImage) -> Option<Self> {
        let ocr = LoadingOcr::get_ocr(image);
        Self::from_raw_ocr(ocr)
    }
    
    pub fn from_raw_ocr(str: String) -> Option<Self> {
        if str.to_lowercase().contains("loading") {
            Some(Loading)
        } else {
            None
        }
    }
}

pub struct LoadingOcr;

impl LoadingOcr {
    pub fn get_ocr(image: &RgbaImage) -> String {
        const X: u32 = 1473;
        const Y: u32 = 930;
        const WIDTH: u32 = 299;
        const HEIGHT: u32 = 87;

        let loading = image.view(X, Y, WIDTH, HEIGHT).to_image();
        loading.save("loading.png").unwrap();

        let mut buffer = vec![];
        let png_encoder = PngEncoder::new(&mut buffer);
        png_encoder
            .write_image(loading.as_raw(), WIDTH, HEIGHT, ExtendedColorType::Rgba8)
            .unwrap();

        let tesseract =
            Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng")).unwrap();

        let loading = tesseract
            .set_image_from_mem(&buffer)
            .unwrap()
            .get_text()
            .unwrap()
            .trim()
            .to_string();

        loading
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading() {
        let image = image::open("loading_screen.png").unwrap();
        let mut image = image.as_rgba8().unwrap();

        let res = LoadingOcr::get_ocr(&mut image);
        // let res = Hp::from_raw_ocr(res);

        println!("{res:#?}");
    }
}
