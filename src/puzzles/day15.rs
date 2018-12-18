use std::error::Error;

use goblinwars::map::MapBuilder;

type Result<T> = ::std::result::Result<T, Box<Error>>;

pub(crate) fn main() -> Result<()> {
    use crate::input_to_string;

    let mut map = MapBuilder::default()
        .build(&input_to_string(15)?)
        .map_err(|e| e.to_string())?;

    let outcome = map
        .run(|_, t| {
            eprint!("\r Time: {}", t);
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    eprint!("\n");
    println!("Part 1: {}", outcome.score);

    Ok(())
}

#[cfg(test)]
mod test {}
