#![allow(dead_code)]

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt;

use std::ops::{Add, BitAnd, BitOr, Mul};

use super::{Instruction, Opcode, Value};

fn program_states(program: &[Instruction], ip: Value, registers: usize) -> Program {
    program
        .iter()
        .cloned()
        .map(|ins| ProgramState::Initial(ins))
        .collect::<Vec<_>>()
        .into()
}

pub(crate) fn decompile(program: &[Instruction], ip: Value) {
    let mut program = program_states(program, ip, 6);

    let mut instruction = 0;

    // Process zero, assuming we can't get here.
    let mut registers = RegisterState::init(6, ip, Variable::Literal(0));
    registers.set(0, Variable::AtLeast(0));

    for i in 0..1000 {
        registers = program.process(&registers, instruction).unwrap();

        // if let ProgramState::Processed(ref p) = &program.state[instruction as usize] {
        //     eprintln!("   {}", p.before);
        //     eprintln!("{}", p.command(instruction as Value));
        //     eprintln!("   {}", p.after);
        // }

        instruction = match registers.nip() {
            Variable::Literal(v) => v,
            b => {
                eprintln!("B {}", b);
                break;
            }
        };
    }

    eprintln!("-----\n\n");
    eprintln!("{}", program);
}

const VARS: &str = "abcdefghijklmnopqrstuvwxyz";

fn var(register: Value) -> String {
    format!("{}", VARS.chars().nth(register as usize).unwrap())
}

#[derive(Debug)]
struct Command {
    label: Option<Value>,

    output: Value,
    action: Action,
}

impl Command {
    fn decompile(instruction: &Instruction, label: Value, register: &RegisterState) -> Self {
        Self {
            label: Some(label),
            output: instruction.output,
            action: Action::from_instruction(instruction, register),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(label) = self.label {
            write!(f, "[{:2}] ", label)?;
        }
        write!(f, "{} = ", var(self.output))?;
        write!(f, "{}", self.action)
    }
}

#[derive(Debug)]
enum Action {
    Unary(Variable),
    Binary(Variable, Op, Variable),
}

impl Action {
    fn from_instruction(instruction: &Instruction, register: &RegisterState) -> Self {
        match instruction.opcode {
            Opcode::Addr => Self::rr(register, instruction),
            Opcode::Addi => Self::ri(register, instruction),
            Opcode::Mulr => Self::rr(register, instruction),
            Opcode::Muli => Self::ri(register, instruction),
            Opcode::Banr => Self::rr(register, instruction),
            Opcode::Bani => Self::ri(register, instruction),
            Opcode::Borr => Self::rr(register, instruction),
            Opcode::Bori => Self::ri(register, instruction),
            Opcode::Setr => Action::Unary(register.get(instruction.input_a)),
            Opcode::Seti => Action::Unary(Variable::Literal(instruction.input_a)),
            Opcode::Gtir => Self::ir(register, instruction),
            Opcode::Gtri => Self::ri(register, instruction),
            Opcode::Gtrr => Self::rr(register, instruction),
            Opcode::Eqir => Self::ir(register, instruction),
            Opcode::Eqri => Self::ri(register, instruction),
            Opcode::Eqrr => Self::rr(register, instruction),
        }
    }

    fn rr(register: &RegisterState, ins: &Instruction) -> Self {
        Action::Binary(
            register.get(ins.input_a),
            ins.opcode.into(),
            register.get(ins.input_b),
        )
    }

    fn ri(register: &RegisterState, ins: &Instruction) -> Self {
        Action::Binary(
            register.get(ins.input_a),
            ins.opcode.into(),
            Variable::Literal(ins.input_b),
        )
    }

    fn ir(register: &RegisterState, ins: &Instruction) -> Self {
        Action::Binary(
            Variable::Literal(ins.input_a),
            ins.opcode.into(),
            register.get(ins.input_b),
        )
    }

    fn process(&self) -> Variable {
        match self {
            Action::Unary(v) => *v,
            Action::Binary(lhs, op, rhs) => op.process(*lhs, *rhs),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Unary(value) => write!(f, "{}", value),
            Action::Binary(lhs, op, rhs) => write!(f, "{} {} {}", lhs, op, rhs),
        }
    }
}

#[derive(Debug)]
enum Op {
    Add,
    Mul,
    Eq,
    Gt,
    Ba,
    Bo,
    Set,
}

impl From<Opcode> for Op {
    fn from(oc: Opcode) -> Self {
        match oc {
            Opcode::Addr => Op::Add,
            Opcode::Addi => Op::Add,
            Opcode::Mulr => Op::Mul,
            Opcode::Muli => Op::Mul,
            Opcode::Banr => Op::Ba,
            Opcode::Bani => Op::Ba,
            Opcode::Borr => Op::Bo,
            Opcode::Bori => Op::Bo,
            Opcode::Setr => Op::Set,
            Opcode::Seti => Op::Set,
            Opcode::Gtir => Op::Gt,
            Opcode::Gtri => Op::Gt,
            Opcode::Gtrr => Op::Gt,
            Opcode::Eqir => Op::Eq,
            Opcode::Eqri => Op::Eq,
            Opcode::Eqrr => Op::Eq,
        }
    }
}

impl Op {
    fn process(&self, lhs: Variable, rhs: Variable) -> Variable {
        match self {
            Op::Add => lhs + rhs,
            Op::Mul => lhs * rhs,
            Op::Ba => lhs & rhs,
            Op::Bo => lhs | rhs,
            Op::Gt => gt(lhs, rhs),
            Op::Eq => eq(lhs, rhs),
            Op::Set => lhs,
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Mul => write!(f, "*"),
            Op::Eq => write!(f, "=="),
            Op::Gt => write!(f, ">"),
            Op::Ba => write!(f, "&"),
            Op::Bo => write!(f, "|"),
            Op::Set => write!(f, "="),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Variable {
    Literal(Value),
    Between(Value, Value),
    AtLeast(Value),
}

impl Variable {
    fn range(self, max: Value) -> Vec<Value> {
        match self {
            Variable::Literal(a) if a < max => vec![a],
            Variable::Literal(_) => Vec::new(),
            Variable::Between(a, b) => (a..cmp::min(b + 1, max)).collect(),
            Variable::AtLeast(a) => (a..max).collect(),
        }
    }

    fn store(self, rhs: Variable) -> Self {
        use self::Variable::*;

        match (self, rhs) {
            (Literal(_), Literal(b)) => Literal(b),
            (AtLeast(a), Literal(b)) => AtLeast(cmp::min(a, b)),
            (Literal(a), AtLeast(b)) => AtLeast(cmp::min(a, b)),
            (Between(lb, lt), Literal(r)) => Between(cmp::min(r, lb), cmp::max(r, lt)),
            (Literal(r), Between(lb, lt)) => Between(cmp::min(r, lb), cmp::max(r, lt)),
            (Between(lb, lt), Between(rb, rt)) => Between(cmp::min(lb, rb), cmp::max(lt, rt)),
            (Between(lb, _), AtLeast(rb)) => AtLeast(cmp::min(lb, rb)),
            (AtLeast(lb), Between(rb, _)) => AtLeast(cmp::min(lb, rb)),
            (AtLeast(lb), AtLeast(rb)) => AtLeast(cmp::min(lb, rb)),
        }
    }
}

impl Add for Variable {
    type Output = Variable;

    fn add(self, rhs: Self) -> Self::Output {
        use self::Variable::*;
        match (self, rhs) {
            // Literal addition
            (Literal(l), Literal(r)) => Literal(l + r),
            (AtLeast(l), Literal(r)) => AtLeast(l + r),
            (Literal(l), AtLeast(r)) => AtLeast(l + r),
            (AtLeast(l), AtLeast(r)) => AtLeast(l + r),

            // Between -> ?
            (Between(lb, lt), Between(rb, rt)) => Between(lb + rb, lt + rt),
            (Between(lb, lt), Literal(v)) => Between(lb + v, lt + v),
            (Literal(v), Between(lb, lt)) => Between(lb + v, lt + v),
            (AtLeast(l), Between(rb, _)) => AtLeast(rb + l),
            (Between(rb, _), AtLeast(l)) => AtLeast(rb + l),
        }
    }
}

impl Mul for Variable {
    type Output = Variable;

    fn mul(self, rhs: Self) -> Self {
        use self::Variable::*;
        match (self, rhs) {
            // Literal addition
            (Literal(l), Literal(r)) => Literal(l * r),

            // Multiplyting by zero gives zero
            (Literal(0), _) => Literal(0),
            (_, Literal(0)) => Literal(0),

            (AtLeast(lb), Literal(r)) => AtLeast(lb * r),
            (Literal(l), AtLeast(rb)) => AtLeast(l * rb),

            (Between(lb, lt), Between(rb, rt)) => Between(lb * rb, lt * rt),
            (Between(lb, lt), Literal(v)) => Between(lb * v, lt * v),
            (Literal(v), Between(lb, lt)) => Between(lb * v, lt * v),
            (AtLeast(lb), Between(rb, _)) => AtLeast(lb * rb),
            (Between(rb, _), AtLeast(lb)) => AtLeast(lb * rb),
            (AtLeast(lb), AtLeast(rb)) => AtLeast(lb * rb),
        }
    }
}

impl BitAnd for Variable {
    type Output = Variable;

    fn bitand(self, rhs: Self) -> Self {
        use self::Variable::*;
        match (self, rhs) {
            // Literal addition
            (Literal(l), Literal(r)) => Literal(l & r),

            // Bitand by zero gives zero
            (Literal(0), _) => Literal(0),
            (_, Literal(0)) => Literal(0),

            // Binary -> Binary
            (Between(0, 1), Between(0, 1)) => Between(0, 1),
            (Between(0, 1), Literal(_)) => Between(0, 1),
            (Literal(_), Between(0, 1)) => Between(0, 1),

            (Between(_, _), _) => AtLeast(0),
            (_, Between(_, _)) => AtLeast(0),
            (AtLeast(_), _) => AtLeast(0),
            (_, AtLeast(_)) => AtLeast(0),
        }
    }
}

fn literal_eq(lhs: Value, rhs: Value) -> Value {
    if lhs == rhs {
        1
    } else {
        0
    }
}

fn eq(lhs: Variable, rhs: Variable) -> Variable {
    use self::Variable::*;
    match (lhs, rhs) {
        // Literals
        (Literal(l), Literal(r)) => Literal(literal_eq(l, r)),

        (AtLeast(l), Literal(r)) => {
            if r < l {
                Literal(0)
            } else {
                Between(0, 1)
            }
        }
        (Literal(l), AtLeast(r)) => {
            if l < r {
                Literal(0)
            } else {
                Between(0, 1)
            }
        }

        (AtLeast(_), AtLeast(_)) => Between(0, 1),

        // Having handled zeros and ones, these must be not equal, therefore 0
        (Between(0, 1), Literal(_)) => Literal(0),
        (Literal(_), Between(0, 1)) => Literal(0),

        (Between(lb, lt), Between(rb, rt)) => {
            if lb > rt || rb > lt {
                Literal(0)
            } else {
                Between(0, 1)
            }
        }

        (Between(_, _), _) => Between(0, 1),
        (_, Between(_, _)) => Between(0, 1),
    }
}

fn literal_gt(lhs: Value, rhs: Value) -> Value {
    if lhs > rhs {
        1
    } else {
        0
    }
}

fn gt(lhs: Variable, rhs: Variable) -> Variable {
    use self::Variable::*;
    match (lhs, rhs) {
        // Literals
        (Literal(l), Literal(r)) => Literal(literal_gt(l, r)),

        (AtLeast(l), Literal(r)) => {
            if l > r {
                Literal(1)
            } else {
                Between(0, 1)
            }
        }

        (Literal(l), AtLeast(r)) => {
            if l > r {
                Between(0, 1)
            } else {
                Literal(0)
            }
        }

        (AtLeast(_), AtLeast(_)) => Between(0, 1),

        (Literal(l), Between(rb, rt)) => {
            if l > rt {
                Literal(1)
            } else if l <= rb {
                Literal(0)
            } else {
                Between(0, 1)
            }
        }

        (Between(rb, rt), Literal(l)) => {
            if rb > l {
                Literal(1)
            } else if rt <= l {
                Literal(0)
            } else {
                Between(0, 1)
            }
        }

        (Between(_, rt), AtLeast(l)) => {
            if l >= rt {
                Literal(0)
            } else {
                Between(0, 1)
            }
        }
        (AtLeast(l), Between(_, rt)) => {
            if l > rt {
                Literal(1)
            } else {
                Between(0, 1)
            }
        }

        (Between(lb, lt), Between(rb, rt)) => {
            if lb > rt {
                Literal(1)
            } else if lt <= rb {
                Literal(0)
            } else {
                Between(0, 1)
            }
        }
    }
}
impl BitOr for Variable {
    type Output = Variable;

    fn bitor(self, rhs: Self) -> Self {
        use self::Variable::*;
        match (self, rhs) {
            // Literal addition
            (Literal(l), Literal(r)) => Literal(l | r),

            // Bitor by zero gives zero
            (Literal(0), Between(0, 1)) => Between(0, 1),
            (Between(0, 1), Literal(0)) => Between(0, 1),

            // Binary -> Binary
            (Between(0, 1), Between(0, 1)) => Between(0, 1),
            (Between(_, _), Literal(_)) => AtLeast(0),
            (Literal(_), Between(_, _)) => AtLeast(0),
            (Between(_, _), Between(_, _)) => AtLeast(0),
            (AtLeast(_), _) => AtLeast(0),
            (_, AtLeast(_)) => AtLeast(0),
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Variable::Literal(value) => write!(f, "{}", value),
            Variable::Between(a, b) => {
                if b - a == 1 {
                    write!(f, "B{}", a)
                } else {
                    write!(f, "R{}-{}", a, b)
                }
            }
            Variable::AtLeast(m) => write!(f, "A{}", m),
        }
    }
}

#[derive(Debug, Clone)]
struct RegisterState {
    registers: Vec<Variable>,
    instruction_register: usize,
}

impl RegisterState {
    fn new(registers: usize, ip: Value) -> Self {
        Self::init(registers, ip, Variable::Literal(0))
    }

    fn init(registers: usize, ip: Value, initial: Variable) -> Self {
        let mut r = Vec::with_capacity(registers);

        for _ in 0..registers {
            r.push(initial);
        }

        Self {
            registers: r,
            instruction_register: ip as usize,
        }
    }

    fn combined(&mut self, other: &Self) {
        for (s, o) in self.registers.iter_mut().zip(other.registers.iter()) {
            *s = s.store(*o);
        }
    }

    fn at(&mut self, label: Variable) {
        self.registers[self.instruction_register] = label;
    }

    fn nip(&self) -> Variable {
        self.registers[self.instruction_register] + Variable::Literal(1)
    }

    fn get(&self, register: Value) -> Variable {
        self.registers[register as usize]
    }

    fn set(&mut self, register: Value, value: Variable) {
        self.registers[register as usize] = value;
    }

    fn store(&mut self, register: Value, value: Variable) {
        let cval = self.registers[register as usize].store(value);

        self.registers[register as usize] = cval;
    }
}

impl fmt::Display for RegisterState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for (i, v) in self.registers.iter().enumerate() {
            if i == self.instruction_register {
                write!(f, "I")?;
            }
            write!(f, "{} ", v)?;
        }
        write!(f, "]")
    }
}

#[derive(Debug, Clone)]
enum ProgramState {
    Initial(Instruction),
    Processed(ProcessedState),
}

impl ProgramState {
    fn process(&self, registers: &RegisterState, label: Value) -> Self {
        let mut before = registers.clone();

        let instruction = match self {
            ProgramState::Initial(i) => i,
            ProgramState::Processed(p) => {
                before.combined(&p.before);
                &p.instruction
            }
        };
        before.at(Variable::Literal(label));
        let cmd = Command::decompile(&instruction, label, &before);
        let mut after = before.clone();
        after.store(cmd.output, cmd.action.process());

        ProgramState::Processed(ProcessedState {
            instruction: instruction.clone(),
            before: before,
            after: after,
        })
    }

    fn next(&self) -> Option<RegisterState> {
        match self {
            ProgramState::Processed(p) => Some(p.after.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct ProcessedState {
    instruction: Instruction,
    before: RegisterState,
    after: RegisterState,
}

impl ProcessedState {
    fn command(&self, label: Value) -> Command {
        Command::decompile(&self.instruction, label, &self.before)
    }
}

#[derive(Debug)]
struct Program {
    state: Vec<ProgramState>,
}

impl From<Vec<ProgramState>> for Program {
    fn from(v: Vec<ProgramState>) -> Self {
        Self { state: v }
    }
}

impl Program {
    fn process(&mut self, registers: &RegisterState, label: Value) -> Option<RegisterState> {
        let idx = label as usize;
        let step = self.state[idx].process(registers, label);
        let next = step.next();
        self.state[idx] = step;
        next
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (label, part) in self.state.iter().enumerate() {
            match part {
                ProgramState::Initial(i) => {
                    writeln!(f, "[{}] NV", label)?;
                }
                ProgramState::Processed(p) => {
                    writeln!(f, "    {}", p.before)?;
                    writeln!(f, "{}", p.command(label as Value))?;
                    writeln!(f, "    {}", p.after)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations() {
        assert_eq!(
            Op::Eq.process(Variable::AtLeast(0), Variable::Literal(72)),
            Variable::Between(0, 1)
        );
    }
}
