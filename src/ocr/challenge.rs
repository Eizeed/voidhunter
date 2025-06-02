use image::{
    codecs::png::PngEncoder, ExtendedColorType, GenericImageView, ImageBuffer, ImageEncoder, Rgba,
};
use tesseract::Tesseract;

#[derive(Debug, Clone)]
pub struct Challenge;

impl Challenge {
    pub fn from_raw_ocr(values: Vec<String>) -> Option<Challenge> {
        let ch_time_1 = values.get(0)?;
        let ch_time_2 = values.get(1)?;
        let ch_defeat = values.get(2)?;

        let mut good_counter = 0;

        if ch_time_1.contains("More")
            || ch_time_1.contains("than")
            || ch_time_1.contains("300s")
            || ch_time_1.contains("remaining")
        {
            // println!("+1 in ch_time_1");
            good_counter += 1;
        } else {
            // println!("-1 in ch_time_1");
            good_counter -= 1;
        }

        if ch_time_2.contains("More")
            || ch_time_2.contains("than")
            || ch_time_2.contains("180s")
            || ch_time_2.contains("remaining")
        {
            // println!("+1 in ch_time_2");
            good_counter += 1;
        } else {
            // println!("-1 in ch_time_2");
            good_counter -= 1;
        }

        if ch_defeat.contains("Defeat")
            || ch_defeat.contains("all")
            || ch_defeat.contains("enemies")
        {
            // println!("+1 in ch_time_3");
            good_counter += 1;
        } else {
            // println!("-1 in ch_time_3");
            good_counter -= 1;
        }

        // println!("Good counter: {}", good_counter);

        if good_counter > 0 {
            Some(Challenge)
        } else {
            None
        }
    }
}

pub struct ChallengeOcr;

impl ChallengeOcr {
    pub fn get_ocr(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Vec<String> {
        const X: u32 = 117;
        const Y1: u32 = 328;
        const Y2: u32 = 366;
        const Y3: u32 = 404;

        const WIDTH1: u32 = 352;
        const WIDTH2: u32 = 260;

        const HEIGHT: u32 = 28;

        let coords = [(Y1, WIDTH1), (Y2, WIDTH1), (Y3, WIDTH2)];

        let mut res = vec![];
        let mut buffer = vec![];

        coords.into_iter().for_each(|(y, w)| {
            let challenge = image.view(X, y, w, HEIGHT).to_image();
            challenge.save(format!("chall-{}-{}.png", y, w)).unwrap();

            let png_encoder = PngEncoder::new(&mut buffer);
            png_encoder
                .write_image(challenge.as_raw(), w, HEIGHT, ExtendedColorType::Rgba8)
                .unwrap();

            let tesseract =
                Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng"))
                    .unwrap();

            let challenge = tesseract
                .set_image_from_mem(&buffer)
                .unwrap()
                .get_text()
                .unwrap()
                .trim()
                .to_string();

            buffer.clear();

            res.push(challenge);
        });

        // res.iter().for_each(|s| {
        //     println!("S: {}", s)
        // });

        res
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use image::RgbaImage;

    use crate::capture::capture_once;

    use super::*;

    #[test]
    fn challenge() {
        let image_buf = image::open("prepare.png").unwrap().to_rgba8();

        let res = ChallengeOcr::get_ocr(&image_buf);
        println!("{res:#?}");
    }
}
