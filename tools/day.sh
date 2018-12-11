day=$1
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )/.."

echo "Preparing for day ${day}"
mkdir -p "$DIR/puzzles/${day}/"
touch "$DIR/puzzles/${day}/input.txt"

if [ ! -e "$DIR/src/puzzles/day$day.rs" ]; then
    cp "$DIR/tools/dayn.rs" "$DIR/src/puzzles/day$day.rs"
    echo "pub mod day${day};" >> "$DIR/src/puzzles/mod.rs"
fi

