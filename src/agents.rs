use image::{
    codecs::png::PngEncoder, ExtendedColorType, GenericImage, ImageBuffer, ImageEncoder, Rgba,
};
use tesseract::Tesseract;

#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
}

impl Agent {
    pub fn from_raw_ocr(agents: &[String]) -> Option<Vec<Option<Agent>>> {
        debug_assert!(agents.len() == 6);

        let mut agent_res = Vec::with_capacity(6);
        for agent in agents.iter() {
            let name = agent
                .split("Lv.")
                .next()
                .expect("The name must be here lol")
                .trim();

            // println!("Name: {name}");

            if Self::NAMES.contains(&name) {
                agent_res.push(Some(Agent {
                    name: name.to_string(),
                }));
                continue;
            }

            if agent == "EMPTY" {
                agent_res.push(None);
                continue;
            }

            return None;
        }

        Some(agent_res)
    }
}

pub struct PickStage;

impl PickStage {
    pub fn get_agent_ocr(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) -> Vec<String> {
        const H1: u32 = 453;
        const H2: u32 = 900;

        const X1: u32 = 367;
        const X2: u32 = 841;
        const X3: u32 = 1314;

        const DIFF: u32 = 131;

        const WIDTH: u32 = 200;
        const HEIGHT: u32 = 60;

        let char_pos = vec![
            (X1, H1),
            (X2, H1),
            (X3, H1),
            (X1 - DIFF, H2),
            (X2 - DIFF, H2),
            (X3 - DIFF, H2),
        ];

        let mut agent_names = Vec::new();
        let mut buffer = Vec::new();

        for (x, y) in char_pos.into_iter() {
            let agent_image = image.sub_image(x, y, WIDTH, HEIGHT).to_image();
            // agent_image.save(format!("char-{}.png", x)).unwrap();

            let png_encoder = PngEncoder::new(&mut buffer);
            png_encoder
                .write_image(
                    agent_image.as_raw(),
                    WIDTH,
                    HEIGHT,
                    ExtendedColorType::Rgba8,
                )
                .unwrap();

            let tesseract =
                Tesseract::new(Some("C:/Program Files/Tesseract-OCR/tessdata"), Some("eng"))
                    .unwrap();

            let agent = tesseract
                .set_image_from_mem(&buffer)
                .unwrap()
                .get_text()
                .unwrap()
                .trim()
                .to_string();

            agent_names.push(agent);
            buffer.clear();
        }

        agent_names
    }
}

impl Agent {
    #[allow(dead_code)]
    pub const NAMES: [&'static str; 33] = [
        "Trigger",
        "Hugo",
        "Yanagi",
        "Lighter",
        "Caesar",
        "Soldier 11",
        "Nekomata",
        "Ben",
        "Anton",
        "Corin",
        "Billy",
        "Harumasa",
        "Miyabi",
        "Pulchra",
        "Piper",
        "Seth",
        "Lucy",
        "Soukaku",
        "Nicole",
        "Anby",
        "Soldier 0 - Anby",
        "Vivian",
        "Evelyn",
        "Astra Yao",
        "Jane",
        "Qingyi",
        "Zhu Yuan",
        "Rina",
        "Ellen",
        "Grace",
        "Burnice",
        "Lycaon",
        "Koleda",
    ];
}
