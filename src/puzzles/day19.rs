use failure::{format_err, Error};
use std::io::prelude::*;

use crate::elfcode::psuedocoder::decompile;
use crate::elfcode::{Instruction, InstructionPointer, Processor};

pub(crate) fn main() -> Result<(), Error> {
    use crate::input;

    let mut lines = input(19)?.lines();

    let ip = lines
        .nth(0)
        .ok_or_else(|| format_err!("No instruction pointer found"))?
        .map_err(Error::from)
        .and_then(|l| l.parse::<InstructionPointer>().map_err(Error::from))?;

    let program = lines
        .map(|lr| {
            lr.map_err(Error::from)
                .and_then(|l| l.parse::<Instruction>().map_err(Error::from))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut processor = Processor::new(program.clone(), 6, ip.into());

    let r = processor
        .run()
        .last()
        .ok_or_else(|| format_err!("No steps ran!"))?;

    println!("Part 1: {}", r.get(0)?);

    eprintln!("{}", decompile(&program, ip.into()));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::elfcode::Register;

    fn example_program() -> Processor {
        let commands = "seti 5 0 1
seti 6 0 2
addi 0 1 0
addr 1 2 3
setr 1 0 0
seti 8 0 4
seti 9 0 5"
            .lines()
            .map(|l| l.parse::<Instruction>().unwrap())
            .collect::<Vec<_>>();

        let ip = "#ip 0".parse::<InstructionPointer>().unwrap();
        Processor::new(commands, 6, ip.into())
    }

    #[test]
    fn example_part1() {
        let mut p = example_program();

        let (i, r) = p.run().enumerate().last().unwrap();

        assert_eq!(i, 4, "Number of operations completed");
        assert_eq!(r, Register::from(vec![7, 5, 6, 0, 0, 9]));
    }

}
