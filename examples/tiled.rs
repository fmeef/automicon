use std::fs::File;

use automicon::automata::{CellularAutomata, Rule};
use rand::Rng;

fn main() {
    let r = rand::rng();
    let data: Vec<u8> = r.random_iter().take(16).collect();
    let len = (data.len() * 8 * 4) as u32;
    let step = (data.len() * 8) as u32;
    let mut v = CellularAutomata::allocate(&data, len, len);
    for x in 0..4 {
        for y in 0..4 {
            let rule = data[x * y];
            println!("rule={rule}");
            v.rule_interlace(&[rule])
                .offset(step * x as u32, step * y as u32)
                .second(true)
                .run()
                .unwrap();
        }
    }

    let image = v.image;
    let mut f = File::create("./test.png").unwrap();
    image.write_to(&mut f, image::ImageFormat::Png).unwrap();
}
