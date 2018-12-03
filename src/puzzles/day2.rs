use std::collections::HashMap;
use std::io::BufRead;

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn box_checksum(boxid: &str) -> (usize, usize) {
    let mut counts = HashMap::new();

    for c in boxid.chars() {
        *counts.entry(c).or_insert(0) += 1;
    }

    let twos = counts.values().any(|&v| v == 2) as usize;
    let threes = counts.values().any(|&v| v == 3) as usize;

    (twos, threes)
}

fn boxes_checksum<'a, T>(boxes: T) -> usize
where
    T: Iterator<Item = &'a str>,
{
    let (twos, threes) = boxes.fold((0, 0), |(twos, threes), boxid| {
        let r = box_checksum(boxid);
        (twos + r.0, threes + r.1)
    });

    twos * threes
}

fn part1() -> Result<usize> {
    use crate::input;

    let data: Vec<String> = input(2)?
        .lines()
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;
    Ok(boxes_checksum(data.iter().map(|s| s.as_str())))
}

fn common_characters(boxid_a: &str, boxid_b: &str) -> Option<String> {
    if boxid_a.len() != boxid_b.len() {
        return None;
    }

    let mut one_mismatch = false;
    let mut common = String::new();

    for (a, b) in boxid_a.chars().zip(boxid_b.chars()) {
        match ((a == b), one_mismatch) {
            (true, _) => {
                common.push(a);
            }
            (false, true) => {
                return None;
            }
            (false, false) => {
                one_mismatch = true;
            }
        }
    }
    Some(common)
}

fn matching_boxes(boxids: Vec<String>) -> Result<String> {
    let n = boxids.len();

    for i in 0..n {
        for j in i + 1..n {
            if let Some(common) = common_characters(&boxids[i], &boxids[j]) {
                return Ok(common);
            }
        }
    }
    Err(From::from("No close ID pairs"))
}

fn part2() -> Result<String> {
    use crate::input;

    let data: Vec<String> = input(2)?
        .lines()
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;
    matching_boxes(data)
}

pub(crate) fn main() -> Result<()> {
    let answer = part1()?;
    println!("Part 1: {}", answer);

    let answer = part2()?;
    println!("Part 2: {}", answer);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn box_ids() -> Vec<&'static str> {
        vec![
            "abcdef", "bababc", "abbcde", "abcccd", "aabcdd", "abcdee", "ababab",
        ]
    }

    #[test]
    fn example_part1() {
        assert_eq!(box_checksum("abcdef"), (0, 0));
        assert_eq!(box_checksum("bababc"), (1, 1));
        assert_eq!(box_checksum("abbcde"), (1, 0));
        assert_eq!(box_checksum("abcccd"), (0, 1));
        assert_eq!(box_checksum("aabcdd"), (1, 0));
        assert_eq!(box_checksum("abcdee"), (1, 0));
        assert_eq!(box_checksum("ababab"), (0, 1));

        assert_eq!(boxes_checksum(box_ids().into_iter()), 12)
    }

    #[test]
    fn answer_part1() {
        assert_eq!(part1().unwrap(), 5704);
    }

    #[test]
    fn example_part2() {
        let boxids: Vec<String> = "abcde
                      fghij
                      klmno
                      pqrst
                      fguij
                      axcye
                      wvxyz"
            .split('\n')
            .map(|s| s.trim().to_string())
            .collect();

        assert_eq!(matching_boxes(boxids).unwrap(), "fgij".to_string());

        assert_eq!(
            common_characters("fghij", "fguij"),
            Some("fgij".to_string())
        )
    }

    #[test]
    fn answer_part2() {
        assert_eq!(part2().unwrap(), "umdryabviapkozistwcnihjqx".to_string())
    }
}
