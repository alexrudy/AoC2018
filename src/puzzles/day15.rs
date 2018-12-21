use std::error::Error;

use goblinwars::map::MapBuilder;
use goblinwars::Game;

type Result<T> = ::std::result::Result<T, Box<Error>>;

pub(crate) fn main() -> Result<()> {
    use crate::input_to_string;

    let mut game = Game::new(
        MapBuilder::default()
            .build(&input_to_string(15)?)
            .map_err(|e| e.to_string())?,
    );

    let outcome = game
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
mod tests {

    use super::*;

    use test::Bencher;

    #[bench]
    fn bench_pathfinder_nocache(b: &mut Bencher) {
        use crate::input_to_string;

        let map = MapBuilder::default()
            .build(&input_to_string(15).unwrap())
            .map_err(|e| e.to_string())
            .unwrap();

        // b.iter(|| {
        //     map.clone().round().play();
        // });
    }

}
