use std::collections::VecDeque;
use std::error::Error;

type Result<T> = ::std::result::Result<T, Box<Error>>;
type Number = usize;

fn get_data() -> Result<VecDeque<Number>> {
    use crate::input;

    let mut data = String::new();
    input(8)?.read_to_string(&mut data)?;
    let data: Result<VecDeque<Number>> = data
        .split_whitespace()
        .map(|d| d.parse::<Number>().map_err(Box::<Error>::from))
        .collect();
    data
}

pub(crate) fn main() -> Result<()> {
    let mut data = get_data()?;

    let root = Node::from_data(&mut data)?;
    println!("Part 1: {}", root.checksum());
    println!("Part 2: {}", root.value());
    Ok(())
}

#[derive(Debug)]
struct NodeHeader {
    children: Number,
    metadata: Number,
}

impl NodeHeader {
    fn from_data(data: &mut VecDeque<Number>) -> Result<Self> {
        Ok(Self {
            children: data
                .pop_front()
                .ok_or_else(|| newerr!("No data for header"))?,
            metadata: data
                .pop_front()
                .ok_or_else(|| newerr!("Not enough entries for header"))?,
        })
    }
}

#[derive(Debug)]
struct Node {
    children: Vec<Node>,
    metadata: Vec<Number>,
}

impl Node {
    fn from_data(data: &mut VecDeque<Number>) -> Result<Self> {
        let header = NodeHeader::from_data(data)?;
        let mut nodes = Vec::with_capacity(header.children);
        let mut meta = Vec::with_capacity(header.metadata);
        for _ in 0..header.children {
            nodes.push(Node::from_data(data)?);
        }

        for _ in 0..header.metadata {
            meta.push(
                data.pop_front()
                    .ok_or_else(|| newerr!("Not enough data for metata"))?,
            );
        }

        Ok(Node {
            children: nodes,
            metadata: meta,
        })
    }

    fn checksum(&self) -> Number {
        self.metadata.iter().sum::<Number>()
            + self.children.iter().map(|c| c.checksum()).sum::<Number>()
    }

    fn value(&self) -> Number {
        if self.children.is_empty() {
            return self.metadata.iter().sum();
        }
        self.metadata
            .iter()
            .map(|m| self.children.get(*m - 1).map(|c| c.value()).unwrap_or(0))
            .sum()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_part1() {
        let tree: Result<VecDeque<Number>> = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2"
            .split_whitespace()
            .map(|n| n.parse::<Number>().map_err(Box::<Error>::from))
            .collect();
        let mut tree = tree.unwrap();

        let root = Node::from_data(&mut tree).unwrap();
        let meta = root.checksum();

        assert_eq!(meta, 138);
    }

    #[test]
    fn answer_part1() {
        let mut data = get_data().unwrap();

        let root = Node::from_data(&mut data).unwrap();
        assert_eq!(root.checksum(), 48496);
    }

    #[test]
    fn example_part2() {
        let tree: Result<VecDeque<Number>> = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2"
            .split_whitespace()
            .map(|n| n.parse::<Number>().map_err(Box::<Error>::from))
            .collect();
        let mut tree = tree.unwrap();

        let root = Node::from_data(&mut tree).unwrap();

        assert_eq!(root.children[0].value(), 33);
        assert_eq!(root.children[1].value(), 0);
        assert_eq!(root.children[1].children[0].value(), 99);

        assert_eq!(root.value(), 66);
    }

    #[test]
    fn answer_part2() {
        let mut data = get_data().unwrap();

        let root = Node::from_data(&mut data).unwrap();
        assert_eq!(root.value(), 32850);
    }
}
