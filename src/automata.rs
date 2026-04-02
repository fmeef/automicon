use image::{GrayImage, Luma};

use crate::{
    error::{Error, Result},
    image::{PIXEL_DARK, PIXEL_LIGHT},
};

pub struct CellularAutomata {
    pub image: GrayImage,
}

pub struct RuleInterlace<'a, 'b, 'c, T> {
    iterations: u32,
    rule: &'b [T],
    data: &'c [u8],
    second: bool,
    offset_x: u32,
    offset_y: u32,
    automata: &'a mut CellularAutomata,
}

impl<'a, 'b, 'c, T> RuleInterlace<'a, 'b, 'c, T>
where
    T: IntoRule,
{
    pub fn second(&mut self, second: bool) -> &mut Self {
        self.second = second;
        self
    }

    pub fn offset(&mut self, x: u32, y: u32) -> &mut Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    pub fn iterations(&mut self, iterations: u32) -> &mut Self {
        self.iterations = iterations;
        self
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
                self.automata
                    .image
                    .put_pixel(i as u32 + self.offset_x, self.offset_y, PIXEL_DARK);
            } else {
                self.automata
                    .image
                    .put_pixel(i as u32 + self.offset_x, self.offset_y, PIXEL_LIGHT);
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.starting_line();

        let mut count = if self.second { 1 } else { 0 };

        for _ in 0..self.iterations {
            for rule in self.rule {
                count += 1;
                let rule = rule.get_rule()?;

                for v in 0..self.data.len() * 8 {
                    let v = v as u32;

                    let a = if v > 0 {
                        *self
                            .automata
                            .image
                            .get_pixel(v - 1 + self.offset_x, count - 1 + self.offset_y)
                    } else {
                        PIXEL_LIGHT
                    };

                    let b = *self
                        .automata
                        .image
                        .get_pixel(v + self.offset_x, count - 1 + self.offset_y);

                    let c = if v + 1 < self.data.len() as u32 * 8 {
                        *self
                            .automata
                            .image
                            .get_pixel(v + 1 + self.offset_x, count - 1 + self.offset_y)
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

                    if count >= self.iterations {
                        return Ok(());
                    }

                    if self.second {
                        let old = *self
                            .automata
                            .image
                            .get_pixel(v + self.offset_x, count - 2 + self.offset_y)
                            == PIXEL_DARK;

                        let new = if old ^ (n == PIXEL_DARK) {
                            PIXEL_DARK
                        } else {
                            PIXEL_LIGHT
                        };
                        self.automata.image.put_pixel(
                            v + self.offset_x,
                            count + self.offset_y,
                            new,
                        );
                    } else {
                        self.automata
                            .image
                            .put_pixel(v + self.offset_x, count + self.offset_y, n);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Rule(u8);

impl Rule {
    fn get(&self, pos: usize) -> bool {
        self.0 >> pos & 0x1 == 1
    }

    pub fn pixel(&self, pos: usize) -> Luma<u8> {
        if self.get(pos) {
            PIXEL_DARK
        } else {
            PIXEL_LIGHT
        }
    }

    pub fn bits(&self) -> [u8; 8] {
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
    pub fn new(data: &[u8]) -> Result<Self> {
        if data.len() >= u32::MAX as usize {
            return Err(Error::InvalidLength);
        }
        let len = data.len() as u32 * 8;
        Ok(Self {
            image: GrayImage::new(len, len),
        })
    }
    pub fn allocate(width: u32, height: u32) -> Self {
        CellularAutomata {
            image: GrayImage::new(width, height),
        }
    }

    pub fn rule_interlace<'a, 'b, 'c, T>(
        &'c mut self,
        rule: &'b [T],
        data: &'a [u8],
    ) -> Result<RuleInterlace<'c, 'b, 'a, T>>
    where
        T: IntoRule,
    {
        if data.len() >= u32::MAX as usize {
            return Err(Error::InvalidLength);
        }

        Ok(RuleInterlace {
            iterations: data.len() as u32 * 8,
            rule,
            data,
            second: false,
            offset_x: 0,
            offset_y: 0,
            automata: self,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::automata::{CellularAutomata, IntoRule, Rule};

    #[test]
    fn inspect() {
        let test = vec![23, 254, 12, 84];

        let mut v = CellularAutomata::new(&test).unwrap();
        let v = v.rule_interlace(&[110], &test).unwrap();

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
}
