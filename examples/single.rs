use std::fs::File;

use automicon::automata::CellularAutomata;
use rand::Rng;

fn main() {
    let r = rand::rng();
    let data: Vec<u8> = r.random_iter().take(16).collect();
    let mut v = CellularAutomata::new(&data);
    v.rule_interlace(&[110]).run().unwrap();

    let image = v.image;
    let mut f = File::create("./test.png").unwrap();
    image.write_to(&mut f, image::ImageFormat::Png).unwrap();
}
