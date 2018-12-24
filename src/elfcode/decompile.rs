#![allow(dead_code)]

use super::{Instruction, Opcode, Value};

const VARS: &str = "abcdefghijklmnopqrstuvwxyz";

fn var(register: Value) -> String {
    format!("{}", VARS.chars().nth(register as usize).unwrap())
}

pub trait Decompile {
    fn decompile(&self) -> String;
}

#[derive(Debug)]
pub(crate) struct LInstruction {
    command: Instruction,
    label: Value,
}

impl LInstruction {
    fn new(command: Instruction, label: Value) -> Self {
        Self { command, label }
    }

    fn f_rr(&self, op: &str) -> String {
        format!(
            "{} = {} {} {}",
            var(self.command.output),
            var(self.command.input_a),
            op,
            var(self.command.input_b)
        )
    }
    fn f_ri(&self, op: &str) -> String {
        format!(
            "{} = {} {} {}",
            var(self.command.output),
            var(self.command.input_a),
            op,
            self.command.input_b
        )
    }
    fn f_ir(&self, op: &str) -> String {
        format!(
            "{} = {} {} {}",
            var(self.command.output),
            self.command.input_a,
            op,
            var(self.command.input_b)
        )
    }
}

fn instruction_to_pseudocode(ins: LInstruction, ip: Value) -> String {
    let cmd = match ins.command.opcode {
        Opcode::Addr => ins.f_rr("+"),
        Opcode::Addi => ins.f_ri("+"),
        Opcode::Mulr => ins.f_rr("*"),
        Opcode::Muli => ins.f_ri("*"),
        Opcode::Banr => ins.f_rr("&"),
        Opcode::Bani => ins.f_ri("&"),
        Opcode::Borr => ins.f_rr("|"),
        Opcode::Bori => ins.f_ri("|"),
        Opcode::Setr => format!("{} = {}", var(ins.command.output), var(ins.command.input_a)),
        Opcode::Seti => format!("{} = {}", var(ins.command.output), ins.command.input_a),
        Opcode::Gtir => ins.f_ir(">"),
        Opcode::Gtri => ins.f_ri(">"),
        Opcode::Gtrr => ins.f_rr(">"),
        Opcode::Eqir => ins.f_ir("=="),
        Opcode::Eqri => ins.f_ri("=="),
        Opcode::Eqrr => ins.f_rr("=="),
    };

    if ins.command.output == ip {
        format!("[{}] jump {}", ins.label, cmd,)
    } else {
        format!("[{}] {}", ins.label, cmd)
    }
}

pub(crate) fn decompile(program: Vec<Instruction>, ip: Value) -> String {
    let program = program
        .into_iter()
        .enumerate()
        .map(|(i, c)| instruction_to_pseudocode(LInstruction::new(c, i as i32), ip))
        .collect::<Vec<_>>();
    program.join("\n")
}

#[derive(Debug)]
struct Command {
    label: Value,

    // If output is None, we are jumping!
    output: Option<Value>,

    action: Action,
}

#[derive(Debug)]
enum Action {
    Unary(Var),
    Binary(Var, Op, Var),
}

#[derive(Debug)]
enum Var {
    Literal(Value),
    Variable(Value),
}

#[derive(Debug)]
enum Op {
    Add,
    Mul,
    Eq,
    Gt,
    Ba,
    Bo,
}
