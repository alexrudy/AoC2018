#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use itertools::Itertools;

pub(crate) fn main() -> Result<(), Box<Error>> {
    use crate::input_to_string;

    let s = input_to_string(16)?;

    let (samples, test_program) = samples_and_program(&s).map_err(|e| e.to_string())?;

    println!(
        "Part 1: {}",
        samples.iter().filter(|s| s.identify().len() >= 3).count()
    );

    let mut decoder = Decoder::new();
    decoder.discover(&samples).map_err(|e| e.to_string())?;

    let test_program: Vec<_> = test_program
        .into_iter()
        .map(|i| decoder.decode(i))
        .collect();

    let state = Register::new();
    let outcome = processor(state, &test_program).map_err(|e| e.to_string())?;
    println!("Part 2: {}", outcome.get(0).map_err(|e| e.to_string())?);

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

type Value = i32;

#[derive(Debug, Fail)]
enum RegisterError {
    #[fail(display = "Invalid Address: {}", _0)]
    InvalidAddress(Value),
}

#[derive(Debug, Fail)]
enum ParseRegisterError {
    #[fail(display = "Not enough register values provided")]
    NotEnoughValues,

    #[fail(display = "Too many register values provided")]
    TooManyValues,

    #[fail(display = "Invalid Value: {}", _0)]
    InvalidValue(ParseIntError),
}

impl From<ParseIntError> for ParseRegisterError {
    fn from(error: ParseIntError) -> Self {
        ParseRegisterError::InvalidValue(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Register {
    memory: [Value; 4],
}

impl Register {
    fn new() -> Self {
        Self { memory: [0; 4] }
    }

    fn from_slice(values: &[Value]) -> Result<Self, ParseRegisterError> {
        if values.len() < 4 {
            return Err(ParseRegisterError::NotEnoughValues);
        } else if values.len() > 4 {
            return Err(ParseRegisterError::TooManyValues);
        }

        let mut register = Self::new();
        for (i, v) in values.iter().take(4).enumerate() {
            register.memory[i] = *v;
        }

        Ok(register)
    }

    fn store(&mut self, address: Value, value: Value) -> Result<(), RegisterError> {
        if address < 0 || address > 3 {
            return Err(RegisterError::InvalidAddress(address));
        }

        self.memory[address as usize] = value;
        Ok(())
    }

    fn get(&self, address: Value) -> Result<Value, RegisterError> {
        if address < 0 || address > 3 {
            return Err(RegisterError::InvalidAddress(address));
        }
        Ok(self.memory[address as usize])
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]",
            (&self.memory).iter().map(|v| v.to_string()).join(",")
        )
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

        Self::from_slice(&values)
    }
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
        Instruction {
            opcode: opcode,
            input_a: self.input_a,
            input_b: self.input_b,
            output: self.output,
        }
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
enum ParseInstructionError {
    #[fail(display = "Not enough register values provided")]
    NotEnoughValues,

    #[fail(display = "Too many register values provided")]
    TooManyValues,

    #[fail(display = "Invalid Value: {}", _0)]
    InvalidValue(ParseIntError),

    #[fail(display = "Invalid Opcode: {}", _0)]
    InvalidOpcode(Value),
}

impl From<ParseIntError> for ParseInstructionError {
    fn from(error: ParseIntError) -> Self {
        ParseInstructionError::InvalidValue(error)
    }
}

impl FromStr for RawInstruction {
    type Err = ParseInstructionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<Value> = s
            .trim()
            .split_whitespace()
            .map(|i| i.trim().parse::<Value>())
            .collect::<Result<Vec<_>, _>>()?;

        let opcode = values[0];
        if opcode > 16 || opcode < 0 {
            return Err(ParseInstructionError::InvalidOpcode(opcode));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Opcode {
    Addr,
    Addi,
    Mulr,
    Muli,
    Banr,
    Bani,
    Borr,
    Bori,
    Setr,
    Seti,
    Gtir,
    Gtri,
    Gtrr,
    Eqir,
    Eqri,
    Eqrr,
}

impl Opcode {
    fn next(self) -> Option<Self> {
        match self {
            Opcode::Addr => Some(Opcode::Addi),
            Opcode::Addi => Some(Opcode::Mulr),
            Opcode::Mulr => Some(Opcode::Muli),
            Opcode::Muli => Some(Opcode::Banr),
            Opcode::Banr => Some(Opcode::Bani),
            Opcode::Bani => Some(Opcode::Borr),
            Opcode::Borr => Some(Opcode::Bori),
            Opcode::Bori => Some(Opcode::Setr),
            Opcode::Setr => Some(Opcode::Seti),
            Opcode::Seti => Some(Opcode::Gtir),
            Opcode::Gtir => Some(Opcode::Gtri),
            Opcode::Gtri => Some(Opcode::Gtrr),
            Opcode::Gtrr => Some(Opcode::Eqir),
            Opcode::Eqir => Some(Opcode::Eqri),
            Opcode::Eqri => Some(Opcode::Eqrr),
            Opcode::Eqrr => None,
        }
    }

    fn all() -> Vec<Opcode> {
        let mut opcodes = Vec::new();

        let mut oc = Opcode::Addr;

        loop {
            opcodes.push(oc);

            let noc = oc.next();
            if noc.is_none() {
                break;
            }
            oc = noc.unwrap();
        }
        opcodes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Instruction {
    opcode: Opcode,
    input_a: Value,
    input_b: Value,
    output: Value,
}

fn gt(a: Value, b: Value) -> Value {
    if a > b {
        1
    } else {
        0
    }
}

fn eq(a: Value, b: Value) -> Value {
    if a == b {
        1
    } else {
        0
    }
}

impl Instruction {
    fn rr<F>(&self, mut register: Register, f: F) -> Result<Register, RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = register.get(self.input_a)?;
        let b = register.get(self.input_b)?;
        register.store(self.output, f(a, b))?;
        Ok(register)
    }

    fn ri<F>(&self, mut register: Register, f: F) -> Result<Register, RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = register.get(self.input_a)?;
        let b = self.input_b;
        register.store(self.output, f(a, b))?;
        Ok(register)
    }

    fn ii<F>(&self, mut register: Register, f: F) -> Result<Register, RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = self.input_a;
        let b = self.input_b;
        register.store(self.output, f(a, b))?;
        Ok(register)
    }

    fn ir<F>(&self, mut register: Register, f: F) -> Result<Register, RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = self.input_a;
        let b = register.get(self.input_b)?;
        register.store(self.output, f(a, b))?;
        Ok(register)
    }

    fn process(&self, register: Register) -> Result<Register, RegisterError> {
        match self.opcode {
            Opcode::Addr => self.rr(register, |a, b| a + b),
            Opcode::Addi => self.ri(register, |a, b| a + b),
            Opcode::Mulr => self.rr(register, |a, b| a * b),
            Opcode::Muli => self.ri(register, |a, b| a * b),
            Opcode::Banr => self.rr(register, |a, b| a & b),
            Opcode::Bani => self.ri(register, |a, b| a & b),
            Opcode::Borr => self.rr(register, |a, b| a | b),
            Opcode::Bori => self.ri(register, |a, b| a | b),
            Opcode::Setr => self.rr(register, |a, _| a),
            Opcode::Seti => self.ii(register, |a, _| a),
            Opcode::Gtir => self.ir(register, gt),
            Opcode::Gtri => self.ri(register, gt),
            Opcode::Gtrr => self.rr(register, gt),
            Opcode::Eqir => self.ir(register, eq),
            Opcode::Eqri => self.ri(register, eq),
            Opcode::Eqrr => self.rr(register, eq),
        }
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

        match instruction.process(self.before) {
            Ok(register) => register == self.after,
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
    Instruction(ParseInstructionError),

    #[fail(display = "No after sample found: {}", _0)]
    MissingAfter(String),
}

impl From<ParseRegisterError> for ParseSampleError {
    fn from(error: ParseRegisterError) -> Self {
        ParseSampleError::Register(error)
    }
}

impl From<ParseInstructionError> for ParseSampleError {
    fn from(error: ParseInstructionError) -> Self {
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
        state = step.process(state)?;
    }
    Ok(state)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn all_opcodes() {
        let opcodes = Opcode::all();
        assert_eq!(opcodes.len(), 16);
    }

    #[test]
    fn register() {
        let register = Register::from_slice(&[3, 2, 1, 0]).unwrap();

        assert_eq!(
            register,
            Register {
                memory: [3, 2, 1, 0]
            }
        );
    }

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
        let register = Register::from_slice(&[3, 0, 0, 2]).unwrap();
        let instruction: RawInstruction = "12 2 3 2".parse().unwrap();

        let outcome = Register::from_slice(&[3, 0, 1, 2]).unwrap();
        assert_eq!(
            instruction.convert(Opcode::Eqir).process(register).unwrap(),
            outcome
        );
    }

}
