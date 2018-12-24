use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use failure::{Error, Fail};

use crate::elfcode::{
    Instruction, Opcode, Register, RegisterConstructionError, RegisterError, Value,
};

pub(crate) fn main() -> Result<(), Error> {
    use crate::input_to_string;

    let s = input_to_string(16)?;

    let (samples, test_program) = samples_and_program(&s)?;

    println!(
        "Part 1: {}",
        samples.iter().filter(|s| s.identify().len() >= 3).count()
    );

    let mut decoder = Decoder::new();
    decoder.discover(&samples)?;

    let test_program: Vec<_> = test_program
        .into_iter()
        .map(|i| decoder.decode(i))
        .collect();

    let state = Register::new(4);
    let outcome = processor(state, &test_program)?;
    println!("Part 2: {}", outcome.get(0)?);

    Ok(())
}

fn samples_and_program(s: &str) -> Result<(Vec<Sample>, Vec<RawInstruction>), ParseSampleError> {
    let mut samples = Vec::new();

    let mut bunch: Vec<&str> = Vec::with_capacity(4);
    let mut blanks = 0;

    let mut lines = s.lines();

    for line in lines.by_ref() {
        if bunch.len() >= 3 {
            let sraw = bunch.join("\n");
            samples.push(sraw.parse()?);
            bunch.clear();
        };

        // We find the end of the samples
        // by waiting for blank lines?
        if blanks > 1 {
            break;
        }

        if line.trim().is_empty() {
            blanks += 1;
            continue;
        }

        blanks = 0;

        bunch.push(line);
    }

    let instructions = lines
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.parse::<RawInstruction>())
        .collect::<Result<Vec<_>, _>>()?;

    Ok((samples, instructions))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RawInstruction {
    opcode: Value,
    input_a: Value,
    input_b: Value,
    output: Value,
}

impl RawInstruction {
    fn convert(&self, opcode: Opcode) -> Instruction {
        Instruction::new(opcode, self.input_a, self.input_b, self.output)
    }
}

impl fmt::Display for RawInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.opcode, self.input_a, self.input_b, self.output
        )
    }
}

#[derive(Debug, Fail)]
enum ParseRawInstructionError {
    #[fail(display = "Invalid Value: {}", _0)]
    InvalidValue(ParseIntError),

    #[fail(display = "Invalid Opcode: {}", _0)]
    InvalidOpcode(Value),
}

impl From<ParseIntError> for ParseRawInstructionError {
    fn from(error: ParseIntError) -> Self {
        ParseRawInstructionError::InvalidValue(error)
    }
}

impl FromStr for RawInstruction {
    type Err = ParseRawInstructionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<Value> = s
            .trim()
            .split_whitespace()
            .map(|i| i.trim().parse::<Value>())
            .collect::<Result<Vec<_>, _>>()?;

        let opcode = values[0];
        if opcode > 16 || opcode < 0 {
            return Err(ParseRawInstructionError::InvalidOpcode(opcode));
        }

        let a = values[1];
        let b = values[2];
        let c = values[3];

        Ok(RawInstruction {
            opcode: opcode,
            input_a: a,
            input_b: b,
            output: c,
        })
    }
}

#[derive(Debug, Fail)]
pub(crate) enum ParseRegisterError {
    #[fail(display = "Not enough register values provided")]
    NotEnoughValues,

    #[fail(display = "Too many register values provided")]
    TooManyValues,

    #[fail(display = "Invalid Value: {}", _0)]
    InvalidValue(ParseIntError),
}

impl From<RegisterConstructionError> for ParseRegisterError {
    fn from(error: RegisterConstructionError) -> Self {
        match error {
            RegisterConstructionError::TooManyValues => ParseRegisterError::TooManyValues,
            RegisterConstructionError::NotEnoughValues => ParseRegisterError::NotEnoughValues,
        }
    }
}

impl From<ParseIntError> for ParseRegisterError {
    fn from(error: ParseIntError) -> Self {
        ParseRegisterError::InvalidValue(error)
    }
}

impl FromStr for Register {
    type Err = ParseRegisterError;

    #[allow(deprecated)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<Value> = s
            .trim()
            .trim_left_matches('[')
            .trim_right_matches(']')
            .split(',')
            .map(|i| i.trim().parse::<Value>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::from(values))
    }
}

#[derive(Debug, Clone)]
struct Sample {
    before: Register,
    instruction: RawInstruction,
    after: Register,
}

impl Sample {
    fn evaluate(&self, opcode: Opcode) -> bool {
        let instruction = self.instruction.convert(opcode);

        let mut register = self.before.clone();
        match instruction.process(&mut register) {
            Ok(()) => register == self.after,
            Err(_) => false,
        }
    }

    fn identify(&self) -> HashSet<Opcode> {
        let mut valid = HashSet::new();
        for oc in &Opcode::all() {
            if self.evaluate(*oc) {
                valid.insert(*oc);
            }
        }
        valid
    }
}

impl fmt::Display for Sample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Before: {}", self.before)?;
        writeln!(f, "{}", self.instruction)?;
        writeln!(f, "After: {}", self.after)
    }
}

#[derive(Debug, Fail)]
enum ParseSampleError {
    #[fail(display = "Register Error: {}", _0)]
    Register(ParseRegisterError),

    #[fail(display = "No before sample found: {}", _0)]
    MissingBefore(String),

    #[fail(display = "No instruction found: {}", _0)]
    MissingInstruction(String),

    #[fail(display = "Instruction Error: {}", _0)]
    Instruction(ParseRawInstructionError),

    #[fail(display = "No after sample found: {}", _0)]
    MissingAfter(String),
}

impl From<ParseRegisterError> for ParseSampleError {
    fn from(error: ParseRegisterError) -> Self {
        ParseSampleError::Register(error)
    }
}

impl From<ParseRawInstructionError> for ParseSampleError {
    fn from(error: ParseRawInstructionError) -> Self {
        ParseSampleError::Instruction(error)
    }
}

impl FromStr for Sample {
    type Err = ParseSampleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let bline = lines
            .next()
            .ok_or_else(|| ParseSampleError::MissingBefore(s.to_string()))?;

        if !bline.starts_with("Before: ") {
            return Err(ParseSampleError::MissingBefore(bline.to_string()));
        }

        let before: Register = bline
            .split(':')
            .nth(1)
            .ok_or_else(|| ParseSampleError::MissingBefore(bline.to_string()))?
            .parse()?;

        let instruction: RawInstruction = lines
            .next()
            .ok_or_else(|| ParseSampleError::MissingInstruction(s.to_string()))?
            .parse()?;

        let aline = lines
            .next()
            .ok_or_else(|| ParseSampleError::MissingAfter(s.to_string()))?;

        if !aline.starts_with("After: ") {
            return Err(ParseSampleError::MissingAfter(aline.to_string()));
        }

        let after: Register = aline
            .split(':')
            .nth(1)
            .ok_or_else(|| ParseSampleError::MissingAfter(aline.to_string()))?
            .parse()?;

        Ok(Sample {
            before: before,
            instruction: instruction,
            after: after,
        })
    }
}

#[derive(Debug, Fail)]
enum DecoderError {
    #[fail(display = "Multiple codes for Opcode: {:?}", _0)]
    MultipleSolutions(Opcode),

    #[fail(display = "Opcode has no solution: {:?}", _0)]
    NoSolution(Value),
}

#[derive(Debug, Clone)]
struct Decoder {
    codes: BTreeMap<Value, Opcode>,
}

impl Decoder {
    fn new() -> Self {
        Self {
            codes: BTreeMap::new(),
        }
    }

    fn decode(&self, instruction: RawInstruction) -> Instruction {
        let opcode = self
            .codes
            .get(&instruction.opcode)
            .expect("Invalid decoding:");
        instruction.convert(*opcode)
    }

    fn discover(&mut self, samples: &[Sample]) -> Result<(), DecoderError> {
        let mut identifiers = BTreeMap::new();
        let mut solution = HashMap::new();

        for sample in samples {
            let sample_candidates = sample.identify();
            if sample_candidates.is_empty() {
                return Err(DecoderError::NoSolution(sample.instruction.opcode));
            }

            let candidates = identifiers
                .entry(sample.instruction.opcode)
                .or_insert_with(|| sample_candidates.clone());

            candidates.retain(|c| sample_candidates.contains(c));
            if candidates.is_empty() {
                return Err(DecoderError::NoSolution(sample.instruction.opcode));
            }
        }

        let mut queue: VecDeque<Value> = identifiers.keys().cloned().collect();

        while !queue.is_empty() {
            let opcode = queue.pop_front().expect("The queue can't be empty!");
            if identifiers[&opcode].len() == 1 {
                let oc = *identifiers[&opcode].iter().nth(0).unwrap();
                if solution.insert(oc, opcode).is_some() {
                    return Err(DecoderError::MultipleSolutions(oc));
                }
            } else if identifiers[&opcode].is_empty() {
                return Err(DecoderError::NoSolution(opcode));
            } else {
                identifiers
                    .get_mut(&opcode)
                    .map(|set| set.retain(|c| !solution.contains_key(c)))
                    .ok_or_else(|| DecoderError::NoSolution(opcode))?;

                queue.push_back(opcode);
            }
        }

        for (key, value) in &solution {
            self.codes.insert(*value, *key);
        }

        Ok(())
    }
}

fn processor(init: Register, instructions: &[Instruction]) -> Result<Register, RegisterError> {
    let mut state = init;
    for step in instructions {
        step.process(&mut state)?;
    }
    Ok(state)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn example_part1() {
        use std::iter::FromIterator;

        let sample: Sample = "Before: [3, 2, 1, 1]
9 2 1 2
After:  [3, 2, 2, 1]"
            .parse()
            .unwrap();

        assert_eq!(sample.identify().len(), 3);

        let opcodes: HashSet<Opcode> =
            HashSet::from_iter([Opcode::Mulr, Opcode::Addi, Opcode::Seti].iter().cloned());

        assert_eq!(sample.identify(), opcodes);
    }

    #[test]
    fn operations() {
        let mut register = Register::from(vec![3, 0, 0, 2]);
        let instruction: RawInstruction = "12 2 3 2".parse().unwrap();

        let outcome = Register::from(vec![3, 0, 1, 2]);

        instruction
            .convert(Opcode::Eqir)
            .process(&mut register)
            .unwrap();

        assert_eq!(register, outcome);
    }

    fn puzzle_input() -> (Vec<Sample>, Vec<RawInstruction>) {
        use crate::input_to_string;
        let s = input_to_string(16).unwrap();
        samples_and_program(&s).unwrap()
    }

    #[test]
    fn answer_part1() {
        let (samples, _) = puzzle_input();
        assert_eq!(
            samples.iter().filter(|s| s.identify().len() >= 3).count(),
            596
        );
    }

    #[test]
    fn answer_part2() {
        let (samples, test_program) = puzzle_input();
        let mut decoder = Decoder::new();
        decoder.discover(&samples).unwrap();

        let test_program: Vec<_> = test_program
            .into_iter()
            .map(|i| decoder.decode(i))
            .collect();

        let state = Register::new(4);
        let outcome = processor(state, &test_program).unwrap();

        assert_eq!(outcome.get(0).unwrap(), 554);
    }

}
