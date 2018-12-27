use failure::{format_err, Error};
use std::io::prelude::*;

use crate::elfcode::{Instruction, InstructionPointer, Processor};
use crate::iterhelper::repeated_element;

pub(crate) fn main() -> Result<(), Error> {
    use crate::input;

    let mut lines = input(21)?.lines();

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
        .monitor_instruction(28)
        .nth(0)
        .ok_or_else(|| format_err!("No steps ran!"))?;

    println!("Part 1: {}", r.get(4)?);

    let mut processor = Processor::new(program.clone(), 6, ip.into());

    println!(
        "Part 2: {}",
        repeated_element(processor.monitor_instruction(28).map(|r| r.get(4).unwrap()))
            .ok_or_else(|| format_err!("No pattern found."))?
            .last()
    );

    Ok(())
}
