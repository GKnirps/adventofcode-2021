use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let lines = parse_lines(&content)?;

    let overlaps = solve_puzzle_one(&lines);
    println!(
        "There are {} overlaps (counting only horizontal and vertical lines).",
        overlaps
    );

    let more_overlaps = solve_puzzle_two(&lines);
    println!(
        "There are {} overlaps when also counting diagonal lines.",
        more_overlaps
    );

    Ok(())
}

fn solve_puzzle_one(lines: &[Line]) -> usize {
    let (size_x, size_y) = map_size(lines);

    // primitive approach: just paint all the lines and count crossings
    let mut map: Vec<u32> = vec![0; size_x * size_y];
    for line in lines.iter().filter(
        |Line {
             from_x,
             from_y,
             to_x,
             to_y,
         }| from_x == to_x || from_y == to_y,
    ) {
        // if not for the filter above, this would draw a rectangle. However, we filter out
        // everything where this does not lead to a line, so we're fine here
        let (from_x, to_x) = minmax(line.from_x, line.to_x);
        let (from_y, to_y) = minmax(line.from_y, line.to_y);
        for y in from_y..=to_y {
            for x in from_x..=to_x {
                map[x + y * size_x] += 1;
            }
        }
    }

    map.iter().filter(|n| **n > 1).count()
}

fn solve_puzzle_two(lines: &[Line]) -> usize {
    let (size_x, size_y) = map_size(lines);

    // again the primitive approach: just paint all the lines and count crossings
    let mut map: Vec<u32> = vec![0; size_x * size_y];

    // first all horizontal and vertical lines (copied from puzzle 1)
    for line in lines.iter().filter(
        |Line {
             from_x,
             from_y,
             to_x,
             to_y,
         }| from_x == to_x || from_y == to_y,
    ) {
        // if not for the filter above, this would draw a rectangle. However, we filter out
        // everything where this does not lead to a line, so we're fine here
        let (from_x, to_x) = minmax(line.from_x, line.to_x);
        let (from_y, to_y) = minmax(line.from_y, line.to_y);
        for y in from_y..=to_y {
            for x in from_x..=to_x {
                map[x + y * size_x] += 1;
            }
        }
    }

    // then all diagonal lines
    for line in lines.iter().filter(
        |Line {
             from_x,
             from_y,
             to_x,
             to_y,
         }| {
            let (xl, xr) = minmax(*from_x, *to_x);
            let (yl, yr) = minmax(*from_y, *to_y);
            xr - xl == yr - yl
        },
    ) {
        let x_step: isize = if line.from_x < line.to_x { 1 } else { -1 };
        let y_step: isize = if line.from_y < line.to_y { 1 } else { -1 };
        let mut dy = line.from_y;
        let mut dx = line.from_x;
        while dy != line.to_y && dx != line.to_x {
            map[dx + dy * size_x] += 1;
            dx = (dx as isize + x_step) as usize;
            dy = (dy as isize + y_step) as usize;
        }
        map[dx + dy * size_x] += 1;
    }

    map.iter().filter(|n| **n > 1).count()
}

fn map_size(lines: &[Line]) -> (usize, usize) {
    (
        lines
            .iter()
            .flat_map(|Line { from_x, to_x, .. }| [*from_x, *to_x])
            .max()
            .unwrap_or(0)
            + 1,
        lines
            .iter()
            .flat_map(|Line { from_y, to_y, .. }| [*from_y, *to_y])
            .max()
            .unwrap_or(0)
            + 1,
    )
}

fn minmax(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Line {
    from_x: usize,
    from_y: usize,
    to_x: usize,
    to_y: usize,
}

fn parse_lines(input: &str) -> Result<Vec<Line>, String> {
    input.lines().map(parse_line).collect()
}

fn parse_line(line: &str) -> Result<Line, String> {
    let (from, to) = line
        .split_once(" -> ")
        .ok_or_else(|| format!("invalid line: '{}'", line))?;
    let (from_x, from_y) = parse_coords(from)?;
    let (to_x, to_y) = parse_coords(to)?;
    Ok(Line {
        from_x,
        from_y,
        to_x,
        to_y,
    })
}

fn parse_coords(s: &str) -> Result<(usize, usize), String> {
    let (x, y) = s
        .split_once(',')
        .ok_or_else(|| format!("invalid coordinates: '{}'", s))?;
    Ok((
        x.parse::<usize>()
            .map_err(|e| format!("unable to parse '{}': {}", x, e))?,
        y.parse::<usize>()
            .map_err(|e| format!("unable to parse '{}': {}", y, e))?,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_line_parses_valid_line() {
        // given
        let input = "6,4 -> 2,0";

        // when
        let result = parse_line(input);

        // then
        let line = result.expect("expected successful parsing");
        assert_eq!(
            line,
            Line {
                from_x: 6,
                from_y: 4,
                to_x: 2,
                to_y: 0,
            }
        );
    }

    const EXAMPLE_INPUT: &str = r"0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2
";
    #[test]
    fn solve_puzzle_one_works_for_example() {
        // given
        let lines = parse_lines(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let overlaps = solve_puzzle_one(&lines);

        // then
        assert_eq!(overlaps, 5);
    }

    #[test]
    fn solve_puzzle_two_works_for_example() {
        // given
        let lines = parse_lines(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let overlaps = solve_puzzle_two(&lines);

        // then
        assert_eq!(overlaps, 12);
    }
}
