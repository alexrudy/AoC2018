use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use regex::Regex;

type Result<T> = ::std::result::Result<T, Box<Error>>;

fn parse_sky() -> Result<Sky> {
    use crate::input;
    let mut s = String::new();
    input(10)?.read_to_string(&mut s)?;
    s.parse()
}

pub(crate) fn main() -> Result<()> {
    let mut sky: Sky = parse_sky()?;

    let t = sky.minimize_area();
    println!("Part 1: \n");
    println!("{}", sky);
    println!("Part 2: \n");
    println!("Appears after {} seconds", t);

    Ok(())
}

type Element = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
struct Coordinate {
    x: Element,
    y: Element,
}

impl Coordinate {
    fn new(x: Element, y: Element) -> Self {
        Self { x, y }
    }
}

impl FromStr for Coordinate {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"<[\s]*([-\d]+),[\s]*([-\d]+)>").unwrap();
        }

        let cap = match RE.captures(s) {
            None => return err!("Can't match Coordinate: {}", s),
            Some(c) => c,
        };

        Ok(Self {
            x: cap[1].parse()?,
            y: cap[2].parse()?,
        })
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Light {
    position: Coordinate,
    velocity: Coordinate,
}

impl Light {
    #[cfg(test)]
    fn new(px: Element, py: Element, vx: Element, vy: Element) -> Self {
        Self {
            position: Coordinate { x: px, y: py },
            velocity: Coordinate { x: vx, y: vy },
        }
    }

    fn advance(&mut self, time: Element) {
        self.position.x += self.velocity.x * time;
        self.position.y += self.velocity.y * time;
    }
}

impl FromStr for Light {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"position=(<[-\d\s,]+>)\s*velocity=(<[-\d\s,]+>)").unwrap();
        }

        let cap = match RE.captures(s) {
            None => return err!("Can't match Light: {}", s),
            Some(c) => c,
        };

        Ok(Self {
            position: cap[1].parse()?,
            velocity: cap[2].parse()?,
        })
    }
}

#[derive(Debug, Clone)]
struct Sky {
    lights: Vec<Light>,
}

impl Sky {
    fn new() -> Self {
        Self { lights: Vec::new() }
    }

    fn advance(&mut self, time: Element) {
        for light in self.lights.iter_mut() {
            light.advance(time);
        }
    }

    fn bbox(&self) -> (Coordinate, Coordinate) {
        let mut xmin = Element::max_value();
        let mut xmax = Element::min_value();
        let mut ymin = Element::max_value();
        let mut ymax = Element::min_value();

        for light in &self.lights {
            if light.position.x > xmax {
                xmax = light.position.x;
            }
            if light.position.x < xmin {
                xmin = light.position.x;
            }

            if light.position.y > ymax {
                ymax = light.position.y;
            }
            if light.position.y < ymin {
                ymin = light.position.y;
            }
        }

        (Coordinate::new(xmax, ymax), Coordinate::new(xmin, ymin))
    }

    fn area(&self) -> Element {
        let (tr, bl) = self.bbox();

        (tr.x - bl.x) * (tr.y - bl.y)
    }

    fn minimize_area(&mut self) -> Element {
        let mut time = 0;
        let mut area = self.area();
        loop {
            self.advance(1);
            time += 1;
            if area < self.area() {
                self.advance(-1);
                time -= 1;
                break;
            }
            area = self.area();
        }
        time
    }
}

impl FromStr for Sky {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        let lights = s
            .lines()
            .map(|l| l.parse())
            .collect::<Result<Vec<Light>>>()?;
        Ok(Sky { lights })
    }
}

impl fmt::Display for Sky {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (tr, bl) = self.bbox();

        let lights = {
            let mut lights = HashSet::new();
            for light in &self.lights {
                lights.insert(light.position);
            }
            lights
        };

        let xsize = tr.x - bl.x;
        let ysize = tr.y - bl.y;

        // println!("{} & {} -> {} x {}", tr, bl, xsize, ysize);

        for y in 0..=ysize {
            let yc = y + bl.y;
            write!(f, " ")?;
            for x in 0..=xsize {
                let xc = x + bl.x;
                let position = Coordinate::new(xc, yc);
                if lights.contains(&position) {
                    write!(f, "#")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, " ")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "position=< 9,  1> velocity=< 0,  2>
position=< 7,  0> velocity=<-1,  0>
position=< 3, -2> velocity=<-1,  1>
position=< 6, 10> velocity=<-2, -1>
position=< 2, -4> velocity=< 2,  2>
position=<-6, 10> velocity=< 2, -2>
position=< 1,  8> velocity=< 1, -1>
position=< 1,  7> velocity=< 1,  0>
position=<-3, 11> velocity=< 1, -2>
position=< 7,  6> velocity=<-1, -1>
position=<-2,  3> velocity=< 1,  0>
position=<-4,  3> velocity=< 2,  0>
position=<10, -3> velocity=<-1,  1>
position=< 5, 11> velocity=< 1, -2>
position=< 4,  7> velocity=< 0, -1>
position=< 8, -2> velocity=< 0,  1>
position=<15,  0> velocity=<-2,  0>
position=< 1,  6> velocity=< 1,  0>
position=< 8,  9> velocity=< 0, -1>
position=< 3,  3> velocity=<-1,  1>
position=< 0,  5> velocity=< 0, -1>
position=<-2,  2> velocity=< 2,  0>
position=< 5, -2> velocity=< 1,  2>
position=< 1,  4> velocity=< 2,  1>
position=<-2,  7> velocity=< 2, -2>
position=< 3,  6> velocity=<-1, -1>
position=< 5,  0> velocity=< 1,  0>
position=<-6,  0> velocity=< 2,  0>
position=< 5,  9> velocity=< 1, -2>
position=<14,  7> velocity=<-2,  0>
position=<-3,  6> velocity=< 2, -1>";

    #[test]
    fn parse_light() {
        let light: Light = "position=< 9,  1> velocity=< 0,  2>".parse().unwrap();

        assert_eq!(light, Light::new(9, 1, 0, 2));

        let light: Light = "position=<-6, 10> velocity=< 2, -2>".parse().unwrap();
        assert_eq!(light, Light::new(-6, 10, 2, -2));
    }

    #[test]
    fn example_part1() {
        let mut sky: Sky = EXAMPLE.parse().unwrap();

        sky.minimize_area();

        assert_eq!(
            format!("{}", sky),
            " #   #  ### 
 #   #   #  
 #   #   #  
 #####   #  
 #   #   #  
 #   #   #  
 #   #   #  
 #   #  ### 
"
        );
    }

    #[test]
    fn example_part2() {
        let mut sky: Sky = EXAMPLE.parse().unwrap();

        let t = sky.minimize_area();

        assert_eq!(t, 3);
    }

}
