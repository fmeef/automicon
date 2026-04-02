use std::fs::File;

use automicon::automata::CellularAutomata;
use image::imageops::resize;
use rand::Rng;

fn main() {
    let r = rand::rng();
    let data: Vec<u8> = r.random_iter().take(16).collect();
    let len = (data.len() * 2) as u32;
    let step = 8 as u32;
    let mut v = CellularAutomata::allocate(len, len);

    for x in 0..4 {
        for y in 0..4 {
            let rule = data[x + 1 * y];
            println!("rule={rule} offset={x},{y}");
            v.rule_interlace(&[rule], &[data[x * y]])
                .unwrap()
                .offset(step * x as u32, step * y as u32)
                .second(false)
                .run()
                .unwrap();
        }
    }

    let image = resize(
        &v.image,
        len * 16,
        len * 16,
        image::imageops::FilterType::Nearest,
    );
    let mut f = File::create("./test.png").unwrap();
    image.write_to(&mut f, image::ImageFormat::Png).unwrap();
}
