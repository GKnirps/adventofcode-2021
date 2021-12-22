use std::cmp::{max, min};
use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let cubes = parse(&content)?;

    let initialization_cubes: Vec<Cube> = cubes
        .iter()
        .filter(|cube| {
            cube.from_x >= -50
                && cube.from_y >= -50
                && cube.from_z >= -50
                && cube.to_x <= 50
                && cube.to_y <= 50
                && cube.to_z <= 50
        })
        .cloned()
        .collect();

    let init_cells_on = on_cells(&initialization_cubes);
    println!("After initialization, {} cubes are on", init_cells_on);

    let cells_on = on_cells(&cubes);
    println!("After a full run, {} cubes are on", cells_on);

    Ok(())
}

fn on_cells(cubes: &[Cube]) -> i64 {
    cubes
        .iter()
        .enumerate()
        .filter(|(_, cube)| cube.on)
        .map(|(i, cube)| non_overwritten_cells(cube, cubes.get((i + 1)..).unwrap_or(&[])))
        .sum()
}

fn non_overwritten_cells(cube: &Cube, overwrites: &[Cube]) -> i64 {
    if let Some(next_overwrite) = overwrites.first() {
        non_overwritten_cubes(cube, next_overwrite)
            .iter()
            .map(|splinter| non_overwritten_cells(&splinter, overwrites.get(1..).unwrap_or(&[])))
            .sum()
    } else {
        cube.volume()
    }
}

fn non_overwritten_cubes(cube: &Cube, overwrite: &Cube) -> HashSet<Cube> {
    let mut result: HashSet<Cube> = HashSet::with_capacity(26);
    if let Some(overwrite) = intersection(cube, overwrite) {
        for (from_x, to_x) in [
            (cube.from_x, overwrite.from_x - 1),
            (overwrite.from_x, overwrite.to_x),
            (overwrite.to_x + 1, cube.to_x),
        ] {
            for (from_y, to_y) in [
                (cube.from_y, overwrite.from_y - 1),
                (overwrite.from_y, overwrite.to_y),
                (overwrite.to_y + 1, cube.to_y),
            ] {
                for (from_z, to_z) in [
                    (cube.from_z, overwrite.from_z - 1),
                    (overwrite.from_z, overwrite.to_z),
                    (overwrite.to_z + 1, cube.to_z),
                ] {
                    let c = Cube {
                        on: cube.on,
                        from_x: from_x,
                        to_x: to_x,
                        from_y: from_y,
                        to_y: to_y,
                        from_z: from_z,
                        to_z: to_z,
                    };
                    if c.from_x != overwrite.from_x
                        || c.to_x != overwrite.to_x
                        || c.from_y != overwrite.from_y
                        || c.to_y != overwrite.to_y
                        || c.from_z != overwrite.from_z
                        || c.to_z != overwrite.to_z
                    {
                        if let Some(valid_cube) = c.valid() {
                            result.insert(valid_cube);
                        }
                    }
                }
            }
        }
    } else {
        result.insert(cube.clone());
    }
    result
}

// intersection of two cubes (always with on=false)
fn intersection(cube1: &Cube, cube2: &Cube) -> Option<Cube> {
    Cube {
        on: false,
        from_x: max(cube1.from_x, cube2.from_x),
        to_x: min(cube1.to_x, cube2.to_x),
        from_y: max(cube1.from_y, cube2.from_y),
        to_y: min(cube1.to_y, cube2.to_y),
        from_z: max(cube1.from_z, cube2.from_z),
        to_z: min(cube1.to_z, cube2.to_z),
    }
    .valid()
}

// It's actually a cuboid.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Cube {
    on: bool,
    from_x: i64,
    to_x: i64,
    from_y: i64,
    to_y: i64,
    from_z: i64,
    to_z: i64,
}

impl Cube {
    fn valid(self) -> Option<Self> {
        if self.from_x > self.to_x || self.from_y > self.to_y || self.from_z > self.to_z {
            None
        } else {
            Some(self)
        }
    }

    fn volume(&self) -> i64 {
        (self.to_x - self.from_x + 1)
            * (self.to_y - self.from_y + 1)
            * (self.to_z - self.from_z + 1)
    }
}

fn parse(input: &str) -> Result<Vec<Cube>, String> {
    input.lines().map(parse_line).collect()
}

fn parse_line(line: &str) -> Result<Cube, String> {
    let (on, coords) = if let Some(coords) = line.strip_prefix("on ") {
        (true, coords)
    } else if let Some(coords) = line.strip_prefix("off ") {
        (false, coords)
    } else {
        return Err(format!(
            "expected 'on ' or 'off ' at start of line '{}'",
            line
        ));
    };
    let mut ranges = coords.splitn(3, ",");
    let (from_x, to_x) = parse_range(
        ranges
            .next()
            .and_then(|r| r.strip_prefix("x="))
            .ok_or_else(|| format!("unable to parse x range in line '{}'", line))?,
    )?;
    let (from_y, to_y) = parse_range(
        ranges
            .next()
            .and_then(|r| r.strip_prefix("y="))
            .ok_or_else(|| format!("unable to parse y range in line '{}'", line))?,
    )?;
    let (from_z, to_z) = parse_range(
        ranges
            .next()
            .and_then(|r| r.strip_prefix("z="))
            .ok_or_else(|| format!("unable to parse z range in line '{}'", line))?,
    )?;

    Ok(Cube {
        on,
        from_x,
        to_x,
        from_y,
        to_y,
        from_z,
        to_z,
    })
}

fn parse_range(input: &str) -> Result<(i64, i64), String> {
    let (from, to) = input
        .split_once("..")
        .ok_or_else(|| format!("Invalid range: '{}'", input))?;
    let from: i64 = from
        .parse()
        .map_err(|e| format!("Unable to parse 'from' part of range {}: {}", from, e))?;
    let to: i64 = to
        .parse()
        .map_err(|e| format!("Unable to parse 'to' part of range {}: {}", from, e))?;
    Ok((from, to))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn on_cells_works_for_first_example() {
        // given
        let cubes = parse(
            r"on x=10..12,y=10..12,z=10..12
on x=11..13,y=11..13,z=11..13
off x=9..11,y=9..11,z=9..11
on x=10..10,y=10..10,z=10..10
",
        )
        .expect("Expected successful parsing");

        // when
        let cells_on = on_cells(&cubes);

        // then
        assert_eq!(cells_on, 39);
    }
}
