use criterion::{criterion_group, criterion_main, Criterion};

use geometry::Direction;

use goblinwars::map::Pathfinder;
use goblinwars::map::{MapBuilder, MapElement};
use goblinwars::sprite::Species;

fn pathfinder_benchmark(c: &mut Criterion) {
    c.bench_function("pathfinder::find_path", |b| {
        let builder = MapBuilder::default();
        let raw_map = include_str!("../examples/pathfinding_benchmark.txt");
        let example_map = builder.build(raw_map).unwrap();

        let pathfinder = Pathfinder::new();
        b.iter(|| {
            for position in example_map.sprites.positions() {
                if let Some(p) = pathfinder.find_path(&example_map, *position) {
                    if let MapElement::Sprite(Species::Goblin) = example_map.element(*position) {
                        assert_eq!(p.direction(), Direction::Up, "{:?}", p);
                    }
                };
            }
        })
    });
}

criterion_group!(benches, pathfinder_benchmark);
criterion_main!(benches);
