use image::{
    codecs::png::PngEncoder,
    imageops::{contrast, grayscale},
    ExtendedColorType, GenericImage, ImageBuffer, ImageEncoder, Luma, Rgba,
};
use tesseract::Tesseract;

#[derive(Debug, Clone, Default)]
pub struct Timer {
    hours: u16,
    minutes: u16,
    seconds: u16,
}

impl Timer {
    pub fn from_raw_ocr(val: &str) -> Option<Self> {
        let mut iter = val.split(':');
        let hours = iter.next()?.trim().parse::<u16>().ok()?;
        let minutes = iter.next()?.trim().parse::<u16>().ok()?;
        let seconds = iter.next()?.trim().parse::<u16>().ok()?;

        Some(Timer {
            hours,
            minutes,
            seconds,
        })
    }

    pub fn as_secs(&self) -> u16 {
        self.hours * 60 * 60 + self.minutes * 60 + self.seconds
    }

    pub fn to_string(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds)
    }
}

pub struct RunStage;

impl RunStage {
    pub fn get_timer_ocr(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        const X_OFFSET: u32 = 1634;
        const Y_OFFSET: u32 = 82;
        const WIDTH: u32 = 126;
        const HEIGHT: u32 = 21;

        let timer = image
            .sub_image(X_OFFSET, Y_OFFSET, WIDTH, HEIGHT)
            .to_image();

        let timer = &contrast(&grayscale(&timer), 200.0);
        // timer.pixels()

        timer.save("ingame_timer.png").unwrap();

        Self::parse_7_dig(timer)
    }

    pub fn parse_7_dig(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> String {
        let mut numbers = String::new();
        let mut segments = Vec::with_capacity(7);
        const WIDTH: u32 = 14;

        // Loop section
        for i in 0..3 {
            // Gap between sections
            let gap = 13;

            // Gap between numbers in section
            let inner_gap = 5;

            // Points to start of section
            let start_x = (i * gap) + (WIDTH * 2 + inner_gap) * i;

            // Loop digits in section
            for k in 0..2 {
                let x = start_x + 1 + (inner_gap + WIDTH) * k;
                // Loop left part
                for num in 0..2 {
                    let x = x;
                    let y = 6 + num * 9;

                    let pixel = image.get_pixel(x, y).0[0];
                    segments.push(pixel > 128);
                }

                // Loop middle part
                for num in 0..3 {
                    let x = x + 7;
                    let y = 1 + num * 9;

                    let pixel = image.get_pixel(x, y).0[0];
                    segments.push(pixel > 128);
                }

                // Loop right part
                for num in 0..2 {
                    let x = x + 13;
                    let y = 6 + num * 9;

                    let pixel = image.get_pixel(x, y).0[0];
                    segments.push(pixel > 128);
                }
                let seg_slice: &[bool; 7] = &segments[0..7].try_into().unwrap();
                let num = parse_segment(seg_slice).map(|n| n.to_string());
                
                let Some(num) = num else {
                    return String::new();
                };

                numbers.push_str(num.as_str());

                segments.clear();
            }

            numbers.push(':');
        }

        numbers
    }
}

pub struct TimerStage;

impl TimerStage {
    pub fn get_timer_ocr(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
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

        timer_str
    }
}

fn parse_segment(segments: &[bool; 7]) -> Option<u8> {
    let idx = NUMBERS
        .iter()
        .enumerate()
        .find(|(_, n)| *n == segments)
        .map(|(i, _)| i as u8);

    idx
}

const ZERO: [bool; 7] = [true, true, true, false, true, true, true];
const ONE: [bool; 7] = [false, false, false, false, false, true, true];
const TWO: [bool; 7] = [false, true, true, true, true, true, false];
const THREE: [bool; 7] = [false, false, true, true, true, true, true];
const FOUR: [bool; 7] = [true, false, false, true, false, true, true];
const FIVE: [bool; 7] = [true, false, true, true, true, false, true];
const SIX: [bool; 7] = [true, true, true, true, true, false, true];
const SEVEN: [bool; 7] = [false, false, true, false, false, true, true];
const EIGHT: [bool; 7] = [true, true, true, true, true, true, true];
const NINE: [bool; 7] = [true, false, true, true, true, true, true];

const NUMBERS: [[bool; 7]; 10] = [ZERO, ONE, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() {
        let image = image::open("ingame_timer.png").unwrap().to_luma8();
        let res = RunStage::parse_7_dig(&image);
        println!("{res}");
    }
}
