use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::BufRead;
use std::str::FromStr;

use failure::{format_err, Error};
use lazy_static::lazy_static;
use regex::Regex;

macro_rules! err {
    ($($tt:tt)*) => { Err(format_err!($($tt)*)) }
}

type Result<T> = ::std::result::Result<T, Error>;

pub(crate) fn main() -> Result<()> {
    use crate::input;

    let instructions: Vec<Instruction> = input(7)?
        .lines()
        .map(|line| {
            line.map_err(Error::from)
                .and_then(|l| l.parse::<Instruction>())
        })
        .collect::<Result<Vec<_>>>()?;

    let steps = steps(&instructions);
    {
        let order: String = topological_sort(&steps)?
            .iter()
            .map(|s| s.id.to_string())
            .collect();
        println!("Part 1: {}", order);
    }
    {
        println!("Part 2: {}", topological_parallel_sort(&steps, 5, 60)?);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Id(String);

impl ToString for Id {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Id {
        Id(s.to_owned())
    }
}

#[derive(Debug, Clone)]
struct Step {
    id: Id,
    dependencies: HashSet<Id>,
}

impl PartialEq for Step {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Step {}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id).reverse()
    }
}

impl Hash for Step {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Step {
    fn new(id: Id) -> Self {
        Self {
            id: id,
            dependencies: HashSet::new(),
        }
    }

    fn depend(&mut self, id: Id) {
        self.dependencies.insert(id);
    }
}

fn steps(instructions: &[Instruction]) -> Vec<Step> {
    let mut constructed = HashMap::new();

    for instruction in instructions {
        constructed
            .entry(instruction.source.clone())
            .or_insert_with(|| Step::new(instruction.source.clone()))
            .depend(instruction.dependent.clone());

        constructed
            .entry(instruction.dependent.clone())
            .or_insert_with(|| Step::new(instruction.dependent.clone()));
    }

    let mut s: Vec<Step> = constructed.drain().map(|(_, s)| s).collect();
    s.sort();
    s.reverse();
    s
}

fn topological_sort(steps: &[Step]) -> Result<Vec<Step>> {
    let mut results = Vec::with_capacity(steps.len());
    let mut completed = HashSet::with_capacity(steps.len());
    let mut heap = BinaryHeap::new();

    for s in steps {
        if s.dependencies.is_empty() {
            heap.push(s);
        }
    }

    while let Some(node) = heap.pop() {
        if completed.insert(node.id.clone()) {
            results.push(node.clone());
        }

        for node in steps.iter().filter(|s| !completed.contains(&s.id)) {
            if node.dependencies.iter().all(|d| completed.contains(d)) {
                heap.push(node);
            }
        }
    }

    if completed.len() != steps.len() {
        return err!("Cyclic graph");
    }

    Ok(results)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
struct Instruction {
    dependent: Id,
    source: Id,
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        //Step C must be finished before step A can begin.
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"Step (?P<dependent>[A-Z]+) must be finished before step (?P<source>[A-Z]+) can begin."
            )
            .unwrap();
        };

        let cap = match RE.captures(s) {
            Some(c) => c,
            None => return err!("Can't match input pattern with {}", s),
        };

        Ok(Self {
            dependent: Id(cap["dependent"].to_string()),
            source: Id(cap["source"].to_string()),
        })
    }
}

#[derive(Debug, Clone)]
struct Job<'s> {
    step: &'s Step,
    start: u32,
}

impl<'s> Job<'s> {
    fn duration(&self) -> u32 {
        u32::from(self.step.id.0.as_bytes()[0]) - 64
    }

    fn until_done(&self, now: u32) -> u32 {
        (self.start + self.duration()).checked_sub(now).unwrap_or(0)
    }
}

impl<'s> PartialEq for Job<'s> {
    fn eq(&self, other: &Self) -> bool {
        self.step.eq(other.step)
    }
}

impl<'s> Eq for Job<'s> {}

impl<'s> PartialOrd for Job<'s> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'s> Ord for Job<'s> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.step.cmp(other.step).reverse()
    }
}

impl<'s> Hash for Job<'s> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.step.hash(state);
    }
}

struct WorkerPool<'j> {
    capacity: usize,
    workers: HashSet<Job<'j>>,
}

impl<'j> WorkerPool<'j> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity,
            workers: HashSet::with_capacity(capacity),
        }
    }

    fn insert(&mut self, item: Job<'j>) -> bool {
        if self.workers.len() < self.capacity {
            return self.workers.insert(item);
        }
        false
    }

    fn contains(&self, item: &Job<'j>) -> bool {
        self.workers.contains(item)
    }

    fn iter(&self) -> impl Iterator<Item = &Job<'j>> {
        self.workers.iter()
    }

    fn remove(&mut self, step: &Step) {
        if let Some(target) = self.workers.iter().find(|j| j.step == step).cloned() {
            self.workers.remove(&target);
        }
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.workers.len()
    }

    fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    fn advance(&self, now: u32) -> u32 {
        self.workers
            .iter()
            .map(|j| j.until_done(now))
            .min()
            .unwrap_or(0)
            + now
    }

    fn launch(&mut self, steps: &'j [Step], completed: &HashSet<Id>, now: u32, offset: u32) {
        for node in steps.iter().filter(|s| !completed.contains(&s.id)) {
            if node.dependencies.iter().all(|d| completed.contains(d)) {
                let j = Job {
                    step: node,
                    start: now + offset,
                };
                if !self.contains(&j) && self.insert(j.clone()) {
                    // println!(
                    //     "[START] {:?} @ {} to {} ({})",
                    //     node.id.0,
                    //     now,
                    //     j.duration() + j.start,
                    //     self.len(),
                    // );
                };

                if !self.contains(&j) {
                    // println!("[PEND ] {:?} @ {} ({})", node.id.0, now, self.len());
                }
            }
        }
    }
}

fn topological_parallel_sort(steps: &[Step], workers: usize, offset: u32) -> Result<u32> {
    let mut results = Vec::with_capacity(steps.len());
    let mut completed = HashSet::with_capacity(steps.len());
    let mut heap: BinaryHeap<&Step> = BinaryHeap::new();

    let mut now = 0;
    let mut workers = WorkerPool::new(workers);

    workers.launch(&steps, &completed, now, offset);

    now = workers.advance(now);

    while !(heap.is_empty() && workers.is_empty()) {
        if let Some(node) = heap.pop() {
            // println!("[DONE ] {:?} @ {}", node.id.0, now);
            workers.remove(node);

            results.push(node.clone());
            // println!(
            //     "{}",
            //     results.iter().map(|n| n.id.0.as_str()).collect::<String>()
            // );
        }

        workers.launch(&steps, &completed, now, offset);

        for job in workers.iter().filter(|j| j.until_done(now) == 0) {
            // println!("[QUEUE] {:?} @ {}", job.step.id.0, now);
            if completed.insert(job.step.id.clone()) {
                heap.push(job.step);
            }
        }

        // print!("{} -> ", now);
        now = workers.advance(now);
        // println!("{}", now);
    }

    if completed.len() != steps.len() {
        return err!("Cyclic graph");
    }

    Ok(now)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "Step C must be finished before step A can begin.
Step C must be finished before step F can begin.
Step A must be finished before step B can begin.
Step A must be finished before step D can begin.
Step B must be finished before step E can begin.
Step D must be finished before step E can begin.
Step F must be finished before step E can begin.";

    #[test]
    fn parse_instruction() {
        let instruction: Instruction = "Step C must be finished before step A can begin."
            .parse()
            .unwrap();
        assert_eq!(
            instruction,
            Instruction {
                source: "A".into(),
                dependent: "C".into()
            }
        )
    }

    #[test]
    fn example_part1() {
        let ins: Vec<Instruction> = EXAMPLE
            .split('\n')
            .map(|s| s.parse::<Instruction>().unwrap())
            .collect();

        let s = steps(&ins);

        let s_order: Vec<Id> = topological_sort(&s)
            .unwrap()
            .iter()
            .map(|s| s.id.clone())
            .collect();
        let s_answer: Vec<Id> = "CABDFE".chars().map(|c| Id(format!("{}", c))).collect();

        assert_eq!(s_order, s_answer)
    }

    #[test]
    fn answer_part1() {
        use crate::input;

        let instructions: Vec<Instruction> = input(7)
            .unwrap()
            .lines()
            .map(|line| {
                line.map_err(Error::from)
                    .and_then(|l| l.parse::<Instruction>())
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();

        let steps = steps(&instructions);
        let order: String = topological_sort(&steps)
            .unwrap()
            .iter()
            .map(|s| s.id.to_string())
            .collect();
        assert_eq!("BHRTWCYSELPUVZAOIJKGMFQDXN", &order);
    }

    #[test]
    fn example_part2() {
        let ins: Vec<Instruction> = EXAMPLE
            .split('\n')
            .map(|s| s.parse::<Instruction>().unwrap())
            .collect();

        let s = steps(&ins);

        assert_eq!(topological_parallel_sort(&s, 2, 0).unwrap(), 15)
    }

    #[test]
    fn answer_part2() {
        use crate::input;

        let instructions: Vec<Instruction> = input(7)
            .unwrap()
            .lines()
            .map(|line| {
                line.map_err(Error::from)
                    .and_then(|l| l.parse::<Instruction>())
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();

        let steps = steps(&instructions);
        let answer = topological_parallel_sort(&steps, 5, 60).unwrap();
        assert_eq!(answer, 959);
    }

}
