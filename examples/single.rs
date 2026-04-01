use std::fs::File;

use automicon::automata::CellularAutomata;
use image::imageops::resize;
use rand::Rng;

fn main() {
    let r = rand::rng();
    let data: Vec<u8> = r.random_iter().take(16).collect();
    let mut v = CellularAutomata::new(&data);
    v.rule_interlace(&[110], &data).run().unwrap();

    let image = resize(
        &v.image,
        data.len() as u32 * 8 * 16,
        data.len() as u32 * 8 * 16,
        image::imageops::FilterType::Nearest,
    );
    let mut f = File::create("./test.png").unwrap();
    image.write_to(&mut f, image::ImageFormat::Png).unwrap();
}
