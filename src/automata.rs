use std::iter;

use image::{GrayImage, ImageBuffer, Luma};

use crate::{
    error::{Error, Result},
    image::{PIXEL_DARK, PIXEL_LIGHT},
};

pub struct CellularAutomata {
    data: Vec<u8>,
    image: GrayImage,
}

#[derive(Debug, Clone, Copy)]
pub struct Rule(u8);

impl Rule {
    fn get(&self, pos: usize) -> bool {
        self.0 >> pos & 0x1 == 1
    }

    fn pixel(&self, pos: usize) -> Luma<u8> {
        if self.get(pos) {
            PIXEL_DARK
        } else {
            PIXEL_LIGHT
        }
    }

    fn bits(&self) -> [u8; 8] {
        let mut out: [u8; 8] = [0; 8];
        for x in 0..8 {
            if self.get(x) {
                out[x] = 1;
            } else {
                out[x] = 0;
            }
        }
        out
    }
}

pub trait IntoRule {
    fn get_rule(&self) -> Result<Rule>;
}

impl IntoRule for [u8; 8] {
    fn get_rule(&self) -> Result<Rule> {
        let mut out = 0;
        for (i, x) in self.iter().enumerate() {
            out = out | (x << i);
        }
        Ok(Rule(out))
    }
}

impl IntoRule for Rule {
    fn get_rule(&self) -> Result<Rule> {
        Ok(*self)
    }
}

impl IntoRule for &[u8] {
    fn get_rule(&self) -> Result<Rule> {
        if self.len() != 8 {
            return Err(Error::InvalidRule);
        }

        let mut out = 0;
        for (i, x) in self.iter().enumerate() {
            out = out | (x << i);
        }
        Ok(Rule(out))
    }
}

impl IntoRule for u8 {
    fn get_rule(&self) -> Result<Rule> {
        Ok(Rule(*self))
    }
}

impl CellularAutomata {
    pub fn new(data: Vec<u8>) -> Self {
        let len = data.len() as u32 * 8;
        CellularAutomata {
            data,
            image: GrayImage::new(len, len),
        }
    }

    #[inline(always)]
    fn inspect(&self, pos: usize) -> bool {
        let i = (pos / 8) as usize;
        let x = (pos % 8) as usize;
        match self.data.get(i) {
            Some(v) => (*v >> x) & 0x1 == 1,
            None => false,
        }
    }

    fn starting_line(&mut self) {
        for i in 0..self.data.len() * 8 {
            if self.inspect(i) {
                self.image.put_pixel(i as u32, 0, PIXEL_DARK);
            } else {
                self.image.put_pixel(i as u32, 0, PIXEL_LIGHT);
            }
        }
    }

    fn rule_interlace<T>(&mut self, iterations: u32, rule: &[T]) -> Result<()>
    where
        T: IntoRule,
    {
        self.starting_line();

        let mut count = 0;

        for _ in 0..iterations {
            for rule in rule {
                count += 1;
                let rule = rule.get_rule()?;

                for v in 0..self.data.len() * 8 {
                    let v = v as u32;

                    let a = if v > 0 {
                        *self.image.get_pixel(v - 1, count - 1)
                    } else {
                        PIXEL_LIGHT
                    };

                    let b = *self.image.get_pixel(v, count - 1);

                    let c = if v + 1 < self.data.len() as u32 * 8 {
                        *self.image.get_pixel(v + 1, count - 1)
                    } else {
                        PIXEL_LIGHT
                    };

                    let n = if a == PIXEL_DARK && b == PIXEL_DARK && c == PIXEL_DARK {
                        rule.pixel(0)
                    } else if a == PIXEL_DARK && b == PIXEL_DARK && c == PIXEL_LIGHT {
                        rule.pixel(1)
                    } else if a == PIXEL_DARK && b == PIXEL_LIGHT && c == PIXEL_DARK {
                        rule.pixel(2)
                    } else if a == PIXEL_DARK && b == PIXEL_LIGHT && c == PIXEL_LIGHT {
                        rule.pixel(3)
                    } else if a == PIXEL_LIGHT && b == PIXEL_DARK && c == PIXEL_DARK {
                        rule.pixel(4)
                    } else if a == PIXEL_LIGHT && b == PIXEL_DARK && c == PIXEL_LIGHT {
                        rule.pixel(5)
                    } else if a == PIXEL_LIGHT && b == PIXEL_LIGHT && c == PIXEL_DARK {
                        rule.pixel(6)
                    } else if a == PIXEL_LIGHT && b == PIXEL_LIGHT && c == PIXEL_LIGHT {
                        rule.pixel(7)
                    } else {
                        PIXEL_LIGHT
                    };

                    if count >= iterations {
                        return Ok(());
                    }

                    self.image.put_pixel(v, count, n);
                }
            }
        }

        Ok(())
    }

    pub fn image(self) -> GrayImage {
        self.image
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use rand::Rng;

    use crate::automata::{CellularAutomata, IntoRule, Rule};

    #[test]
    fn inspect() {
        let test = vec![23, 254, 12, 84];
        let v = CellularAutomata::new(test);

        let check = vec![
            true, true, true, false, true, false, false, false, false, true, true, true, true,
            true, true, true, false, false, true, true, false, false, false, false, false, false,
            true, false, true, false, true, false,
        ];

        for c in 0..check.len() {
            assert!(v.inspect(c) == check[c]);
        }
    }

    #[test]
    fn into_rule() {
        let r = 110 as u8;
        let r: Rule = r.get_rule().unwrap();

        assert_eq!(r.bits(), [0, 1, 1, 1, 0, 1, 1, 0]);
    }

    #[test]
    fn image() {
        let r = rand::rng();
        let data: Vec<u8> = r.random_iter().take(16).collect();
        let len = data.len() as u32 * 8;
        let mut v = CellularAutomata::new(data);
        v.rule_interlace(len, &[110]).unwrap();

        let image = v.image();
        let mut f = File::create("./test.png").unwrap();
        image.write_to(&mut f, image::ImageFormat::Png).unwrap();
    }
}
