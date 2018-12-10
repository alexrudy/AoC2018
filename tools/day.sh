day=$1
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )/.."

echo "Preparing for day ${day}"
mkdir -p "$DIR/puzzles/${day}/"
touch -p "$DIR/puzzles/${day}/input.txt"

if [ ! -e "$DIR/src/puzzles/day$day.rs" ]; then
    cp "$DIR/dayn.rs" "$DIR/src/puzzles/day$day.rs"
fi
