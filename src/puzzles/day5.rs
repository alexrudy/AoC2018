use failure::Error;

type Result<T> = ::std::result::Result<T, Error>;

pub(crate) fn main() -> Result<()> {
    let polymer = read_polymer()?;
    println!("Part 1: {}", part1(&polymer)?);
    println!("Part 2: {}", part2(&polymer)?);
    Ok(())
}

fn read_polymer() -> Result<String> {
    use crate::input;
    let mut polymer = String::new();
    input(5)?.read_to_string(&mut polymer)?;
    Ok(polymer)
}

fn reacts(a: char, b: char) -> bool {
    a.eq_ignore_ascii_case(&b) && (a.is_ascii_lowercase() != b.is_ascii_lowercase())
}

fn process(original: &str) -> String {
    let mut output = Vec::new();
    for c in original.chars() {
        if !output.is_empty() && reacts(output[output.len() - 1], c) {
            output.pop();
        } else {
            output.push(c);
        }
    }

    output.iter().collect()
}

fn part1(polymer: &str) -> Result<usize> {
    Ok(process(polymer).len())
}

static ASCII_LOWER: &str = "abcdefghijklmnopqrstuvwxyz";

fn process_removed(original: &str, remove: char) -> String {
    let mut output = Vec::new();
    for c in original.chars() {
        if c.eq_ignore_ascii_case(&remove) {

        } else if !output.is_empty() && reacts(output[output.len() - 1], c) {
            output.pop();
        } else {
            output.push(c);
        }
    }

    output.iter().collect()
}

fn part2(polymer: &str) -> Result<usize> {
    Ok(ASCII_LOWER
        .chars()
        .map(|c| process_removed(polymer, c).len())
        .min()
        .unwrap())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_part1() {
        assert_eq!(process("aA"), "".to_string());
        assert_eq!(process("abBA"), "".to_string());
        assert_eq!(process("abAB"), "abAB".to_string());
        assert_eq!(process("aabAAB"), "aabAAB".to_string());

        assert_eq!(process("dabAcCaCBAcCcaDA"), "dabCBAcaDA".to_string());
    }

    #[test]
    fn answer_part1() {
        let polymer = read_polymer().unwrap();
        assert_eq!(part1(&polymer).unwrap(), 10384);
    }

    #[test]
    fn example_part2() {
        assert_eq!(part2("dabAcCaCBAcCcaDA").unwrap(), 4);
    }
}
