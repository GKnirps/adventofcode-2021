use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let floor = parse(&content)?;

    let steps = run_until_deadlock(floor);
    println!("{} steps until no sea cucumber moves anymore", steps);

    Ok(())
}

fn run_until_deadlock(mut floor: Floor) -> u64 {
    let mut counter = 0;
    let mut moved = true;
    while moved {
        let (f, m) = run_step(floor);
        floor = f;
        moved = m;
        counter += 1;
    }
    counter
}

// run a movement step (for both directions), return the new constellation and whether any sea
// cucumber moved
fn run_step(floor: Floor) -> (Floor, bool) {
    let mut moved = false;
    let width = floor.width;
    let mut buffer: Vec<Tile> = vec![Tile::Empty; floor.tiles.len()];

    for (i, _) in floor
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| **t == Tile::East)
    {
        let x = i % width;
        let y = i / width;
        let target = (x + 1) % width + y * width;
        if floor.tiles[target] == Tile::Empty {
            buffer[target] = Tile::East;
            moved = true;
        } else {
            buffer[i] = Tile::East;
        }
    }
    for (i, _) in floor
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| **t == Tile::South)
    {
        buffer[i] = Tile::South;
    }
    let tiles = buffer;
    let mut buffer = floor.tiles;
    buffer.fill(Tile::Empty);

    for (i, _) in tiles.iter().enumerate().filter(|(_, t)| **t == Tile::South) {
        let target = (i + width) % tiles.len();
        if tiles[target] == Tile::Empty {
            buffer[target] = Tile::South;
            moved = true;
        } else {
            buffer[i] = Tile::South;
        }
    }
    for (i, _) in tiles.iter().enumerate().filter(|(_, t)| **t == Tile::East) {
        buffer[i] = tiles[i];
    }
    (
        Floor {
            width,
            tiles: buffer,
        },
        moved,
    )
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Tile {
    Empty,
    East,
    South,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Floor {
    width: usize,
    tiles: Vec<Tile>,
}

fn parse(input: &str) -> Result<Floor, String> {
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "expected non-empty input".to_owned())?
        .chars()
        .count();
    let tiles = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '.' => Ok(Tile::Empty),
            '>' => Ok(Tile::East),
            'v' => Ok(Tile::South),
            _ => Err(format!("unknown floor tile: '{}'", c)),
        })
        .collect::<Result<Vec<Tile>, String>>()?;

    if tiles.len() % width != 0 {
        Err("floor map is not a rectangle".to_owned())
    } else {
        Ok(Floor { width, tiles })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn run_until_deadlock_works_for_example() {
        // given
        let floor = parse(
            r"v...>>.vv>
.vv>>.vv..
>>.>v>...v
>>v>>.>.v.
v>v.vv.v..
>.>>..v...
.vv..>.>v.
v.v..>>v.v
....v..v.>
",
        )
        .expect("expected successful parsing");

        // when
        let steps = run_until_deadlock(floor);

        // then
        assert_eq!(steps, 58);
    }
}
