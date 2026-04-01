use std::fs::File;

use automicon::automata::CellularAutomata;
use rand::Rng;

fn main() {
    let r = rand::rng();
    let data: Vec<u8> = r.random_iter().take(16).collect();
    let len = data.len() as u32 * 8;
    let mut v = CellularAutomata::new(&data);
    v.rule_interlace(len, &[110], true).unwrap();

    let image = v.image();
    let mut f = File::create("./test.png").unwrap();
    image.write_to(&mut f, image::ImageFormat::Png).unwrap();
}
