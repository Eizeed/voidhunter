use image::{GenericImageView, RgbaImage};

pub mod agents;
pub mod challenge;
pub mod confirm;
pub mod frontier;
pub mod hp;
pub mod loading;
pub mod pause;
pub mod timer;

pub fn is_black_screen(image: &RgbaImage) -> bool {
    let width = image.width() / 4;
    let height = image.height() - 100;
    for i in 0..4 {
        let view = image.view(width * i, 0, width, height);
        if view
            .pixels()
            .all(|p| p.2 .0[0] == 0 && p.2 .0[1] == 0 && p.2 .0[2] == 0 && p.2 .0[3] == 255)
        {
            return true;
        }
    }

    return false;
}
