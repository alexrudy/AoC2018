use std::convert::TryFrom;
use std::fmt;
use std::num::{ParseIntError, TryFromIntError};
use std::str::FromStr;

use failure::Fail;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub(crate) mod decompile;

pub(crate) type Value = i32;

#[derive(Debug, Fail)]
pub(crate) enum RegisterError {
    #[fail(display = "Invalid Address: {}", _0)]
    InvalidAddress(Value),

    #[fail(display = "Invalid Value")]
    InvalidValue,
}

impl From<TryFromIntError> for RegisterError {
    fn from(_error: TryFromIntError) -> Self {
        RegisterError::InvalidValue
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Register {
    memory: Vec<Value>,
}

impl From<Vec<Value>> for Register {
    fn from(sl: Vec<Value>) -> Self {
        Self { memory: sl }
    }
}

impl Register {
    pub(crate) fn new(size: usize) -> Self {
        let mut memory = Vec::with_capacity(size);
        for _ in 0..size {
            memory.push(0);
        }
        Self { memory }
    }

    pub(crate) fn store(&mut self, address: Value, value: Value) -> Result<(), RegisterError> {
        if address < 0 || address > self.memory.len() as i32 {
            return Err(RegisterError::InvalidAddress(address));
        }

        self.memory[address as usize] = value;
        Ok(())
    }

    pub(crate) fn get(&self, address: Value) -> Result<Value, RegisterError> {
        if address < 0 || address > self.memory.len() as i32 {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Opcode {
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

    pub(crate) fn all() -> Vec<Opcode> {
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

#[derive(Debug, Fail)]
pub(crate) enum ParseOpcodeError {
    #[fail(display = "Invalid Opcode: {}", _0)]
    InvalidOpcode(String),
}

impl FromStr for Opcode {
    type Err = ParseOpcodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let oc = match s {
            "addr" => Opcode::Addr,
            "addi" => Opcode::Addi,
            "mulr" => Opcode::Mulr,
            "muli" => Opcode::Muli,
            "banr" => Opcode::Banr,
            "bani" => Opcode::Bani,
            "borr" => Opcode::Borr,
            "bori" => Opcode::Bori,
            "setr" => Opcode::Setr,
            "seti" => Opcode::Seti,
            "gtir" => Opcode::Gtir,
            "gtri" => Opcode::Gtri,
            "gtrr" => Opcode::Gtrr,
            "eqir" => Opcode::Eqir,
            "eqrr" => Opcode::Eqrr,
            "eqri" => Opcode::Eqri,
            _ => return Err(ParseOpcodeError::InvalidOpcode(s.to_string())),
        };
        Ok(oc)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Instruction {
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
    pub(crate) fn new(opcode: Opcode, input_a: Value, input_b: Value, output: Value) -> Self {
        Self {
            opcode,
            input_a,
            input_b,
            output,
        }
    }

    fn rr<F>(&self, register: &mut Register, f: F) -> Result<(), RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = register.get(self.input_a)?;
        let b = register.get(self.input_b)?;
        register.store(self.output, f(a, b))?;
        Ok(())
    }

    fn ri<F>(&self, register: &mut Register, f: F) -> Result<(), RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = register.get(self.input_a)?;
        let b = self.input_b;
        register.store(self.output, f(a, b))?;
        Ok(())
    }

    fn ii<F>(&self, register: &mut Register, f: F) -> Result<(), RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = self.input_a;
        let b = self.input_b;
        register.store(self.output, f(a, b))?;
        Ok(())
    }

    fn ir<F>(&self, register: &mut Register, f: F) -> Result<(), RegisterError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let a = self.input_a;
        let b = register.get(self.input_b)?;
        register.store(self.output, f(a, b))?;
        Ok(())
    }

    pub(crate) fn process(&self, register: &mut Register) -> Result<(), RegisterError> {
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

#[derive(Debug, Fail)]
pub(crate) enum ParseInstructionError {
    #[fail(display = "{}", _0)]
    InvalidOpcode(ParseOpcodeError),

    #[fail(display = "Invalid Register")]
    InvalidRegister(ParseIntError),
}

impl From<ParseOpcodeError> for ParseInstructionError {
    fn from(error: ParseOpcodeError) -> Self {
        ParseInstructionError::InvalidOpcode(error)
    }
}

impl From<ParseIntError> for ParseInstructionError {
    fn from(error: ParseIntError) -> Self {
        ParseInstructionError::InvalidRegister(error)
    }
}

impl FromStr for Instruction {
    type Err = ParseInstructionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<&str> = s.trim().split_whitespace().collect();

        let opcode: Opcode = values[0].parse()?;

        let values = values
            .iter()
            .skip(1)
            .map(|v| v.parse::<Value>())
            .collect::<Result<Vec<_>, _>>()?;

        let a = values[0];
        let b = values[1];
        let c = values[2];

        Ok(Instruction {
            opcode: opcode,
            input_a: a,
            input_b: b,
            output: c,
        })
    }
}

#[derive(Debug, Fail)]
pub(crate) enum ProgramError {
    #[fail(display = "Register error: {}", _0)]
    Register(#[cause] RegisterError),

    #[fail(display = "Program halted")]
    Halted,
}

impl From<RegisterError> for ProgramError {
    fn from(error: RegisterError) -> Self {
        ProgramError::Register(error)
    }
}

impl From<TryFromIntError> for ProgramError {
    fn from(error: TryFromIntError) -> Self {
        ProgramError::Register(RegisterError::from(error))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Processor {
    commands: Vec<Instruction>,
    register: Register,
    instruction_pointer: Value,
}

impl Processor {
    pub(crate) fn new(
        commands: Vec<Instruction>,
        registers: usize,
        instruction_pointer: Value,
    ) -> Self {
        Self {
            commands,
            register: Register::new(registers),
            instruction_pointer,
        }
    }

    pub(crate) fn step(&mut self) -> Result<(), ProgramError> {
        let ip = usize::try_from(self.register.get(self.instruction_pointer)?)?;

        if ip >= self.commands.len() {
            return Err(ProgramError::Halted);
        }

        let instruction = self.commands[ip];
        instruction.process(&mut self.register)?;
        self.register.store(
            self.instruction_pointer,
            self.register.get(self.instruction_pointer)? + 1,
        )?;
        Ok(())
    }

    pub(crate) fn run(&mut self) -> Process {
        Process { processor: self }
    }
}

#[derive(Debug)]
pub(crate) struct Process<'p> {
    processor: &'p mut Processor,
}

impl<'p> Iterator for Process<'p> {
    type Item = Register;

    fn next(&mut self) -> Option<Self::Item> {
        if self.processor.step().is_err() {
            return None;
        }
        Some(self.processor.register.clone())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InstructionPointer(Value);

#[derive(Debug, Fail)]
pub(crate) enum ParseIPError {
    #[fail(display = "Invalid Number: {}", _0)]
    InvalidNumber(String),

    #[fail(display = "Invalid Pattern: {}", _0)]
    InvalidPattern(String),
}

impl From<InstructionPointer> for Value {
    fn from(ip: InstructionPointer) -> Self {
        ip.0
    }
}

impl FromStr for InstructionPointer {
    type Err = ParseIPError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"#ip (\d+)").unwrap();
        }
        match RE.captures(s) {
            Some(c) => {
                Ok(InstructionPointer(c[1].parse::<Value>().map_err(|_| {
                    ParseIPError::InvalidNumber(c[1].to_string())
                })?))
            }
            None => Err(ParseIPError::InvalidPattern(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_opcodes() {
        let opcodes = Opcode::all();
        assert_eq!(opcodes.len(), 16);
    }

    #[test]
    fn register() {
        let register = Register::from([3, 2, 1, 0].to_vec());

        assert_eq!(
            register,
            Register {
                memory: vec![3, 2, 1, 0]
            }
        );
    }

    #[test]
    fn instructions() {
        let i: Instruction = "seti 5 0 1".parse().unwrap();
        assert_eq!(i, Instruction::new(Opcode::Seti, 5, 0, 1));
    }
}
