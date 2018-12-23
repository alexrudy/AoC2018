use failure::Error;

type Result<T> = ::std::result::Result<T, Error>;

pub(crate) fn main() -> Result<()> {
    let recipies = evovle(306_281);
    let part1: String = recipies.iter().map(|d| format!("{}", d)).collect();
    println!("Part 1: {}", part1);

    println!(
        "Part 2: {}",
        Scoreboard::new().hunt(vec![3, 0, 6, 2, 8, 1]).count()
    );

    Ok(())
}

type Recipe = usize;
type Elf = usize;

struct Scoreboard {
    board: Vec<Recipe>,
    left: Elf,
    right: Elf,
}

impl Scoreboard {
    fn new() -> Self {
        Self {
            board: vec![3, 7],
            left: 0,
            right: 1,
        }
    }

    fn cook(&mut self) {
        let left_recipe = self.board[self.left];
        let right_recipe = self.board[self.right];
        self.board.extend(new_recipes(left_recipe, right_recipe));
        self.left = (self.left + 1 + left_recipe) % self.board.len();
        self.right = (self.right + 1 + right_recipe) % self.board.len();
    }

    fn len(&self) -> usize {
        self.board.len()
    }

    fn iter(&mut self) -> ScoreboardIter {
        ScoreboardIter {
            position: 0,
            scoreboard: self,
        }
    }

    fn hunt(&mut self, pattern: Vec<Recipe>) -> ScoreboardPattern {
        ScoreboardPattern {
            position: 0,
            scoreboard: self,
            pattern: pattern,
        }
    }
}

struct ScoreboardIter<'s> {
    position: usize,
    scoreboard: &'s mut Scoreboard,
}

impl<'a> Iterator for ScoreboardIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.position += 1;
        while self.scoreboard.len() < self.position {
            self.scoreboard.cook();
        }
        Some(self.scoreboard.board[self.position - 1])
    }
}

struct ScoreboardPattern<'s> {
    position: usize,
    scoreboard: &'s mut Scoreboard,
    pattern: Vec<Recipe>,
}

impl<'a> Iterator for ScoreboardPattern<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.scoreboard.len() < (self.position + self.pattern.len()) {
            self.scoreboard.cook();
        }
        if &(self.scoreboard.board[self.position..(self.position + self.pattern.len())])
            == self.pattern.as_slice()
        {
            return None;
        }
        self.position += 1;

        Some(self.scoreboard.board[self.position - 1])
    }
}

fn new_recipes(left: Recipe, right: Recipe) -> Vec<Recipe> {
    let total = left + right;
    total
        .to_string()
        .chars()
        .map(|c| format!("{}", c).parse::<Recipe>().unwrap())
        .collect()
}

fn evovle(iterations: usize) -> Vec<Recipe> {
    Scoreboard::new().iter().skip(iterations).take(10).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cooking() {
        assert_eq!(new_recipes(3, 7), vec![1, 0]);
    }

    #[test]
    fn scoreboard() {
        let mut scoreboard = Scoreboard::new();
        scoreboard.cook();
        assert_eq!(scoreboard.board, vec![3, 7, 1, 0]);
        scoreboard.cook();
        assert_eq!(scoreboard.board, vec![3, 7, 1, 0, 1, 0]);

        for _ in 2..15 {
            scoreboard.cook();
        }

        assert_eq!(
            scoreboard.board,
            vec![3, 7, 1, 0, 1, 0, 1, 2, 4, 5, 1, 5, 8, 9, 1, 6, 7, 7, 9, 2]
        );
    }

    #[test]
    fn example_part1() {
        assert_eq!(evovle(9), vec![5, 1, 5, 8, 9, 1, 6, 7, 7, 9]);
        assert_eq!(evovle(5), vec![0, 1, 2, 4, 5, 1, 5, 8, 9, 1]);
        assert_eq!(evovle(18), vec![9, 2, 5, 1, 0, 7, 1, 0, 8, 5]);
        assert_eq!(evovle(2018), vec![5, 9, 4, 1, 4, 2, 9, 8, 8, 2]);
    }

    #[test]
    fn example_part2() {
        assert_eq!(Scoreboard::new().hunt(vec![5, 1, 5, 8, 9]).count(), 9);
        assert_eq!(Scoreboard::new().hunt(vec![0, 1, 2, 4, 5]).count(), 5);
        assert_eq!(Scoreboard::new().hunt(vec![9, 2, 5, 1, 0]).count(), 18);
        assert_eq!(Scoreboard::new().hunt(vec![5, 9, 4, 1, 4]).count(), 2018);
    }

}
