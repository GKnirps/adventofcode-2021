use std::env;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Instruction {
    dir: Direction,
    value: i32,
}

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let instructions = parse_instructions(&content)?;

    let (p1x, p1y) = solve_puzzle_one(&instructions);
    println!("Position is ({}, {}), product is {}", p1x, p1y, p1x * p1y);

    let (p2x, p2y) = solve_puzzle_two(&instructions);
    println!(
        "Position with aim is ({}, {}), product is {}",
        p2x,
        p2y,
        p2x * p2y
    );

    Ok(())
}

fn solve_puzzle_one(instructions: &[Instruction]) -> (i32, i32) {
    instructions
        .iter()
        .fold((0, 0), |(x, y), Instruction { dir, value }| match dir {
            Direction::Horizontal => (x + value, y),
            Direction::Vertical => (x, y + value),
        })
}

fn solve_puzzle_two(instructions: &[Instruction]) -> (i32, i32) {
    let (x, y, _) = instructions.iter().fold(
        (0, 0, 0),
        |(x, y, aim), Instruction { dir, value }| match dir {
            Direction::Vertical => (x, y, aim + value),
            Direction::Horizontal => (x + value, y + aim * value, aim),
        },
    );
    (x, y)
}

fn parse_instructions(content: &str) -> Result<Vec<Instruction>, String> {
    content.lines().map(parse_instruction).collect()
}

fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let (dir, value) = line
        .split_once(" ")
        .ok_or_else(|| format!("Unable to parse instruction '{}'", line))?;
    let value = value
        .parse::<i32>()
        .map_err(|e| format!("Unable to parse value of instruction '{}': {}", line, e))?;
    match dir {
        "forward" => Ok(Instruction {
            dir: Direction::Horizontal,
            value,
        }),
        "down" => Ok(Instruction {
            dir: Direction::Vertical,
            value,
        }),
        "up" => Ok(Instruction {
            dir: Direction::Vertical,
            value: -value,
        }),
        _ => Err(format!("Unknown direction: '{}'", dir)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_instruction_parses_correctly() {
        assert_eq!(
            parse_instruction("forward 9"),
            Ok(Instruction {
                dir: Direction::Horizontal,
                value: 9
            })
        );
        assert_eq!(
            parse_instruction("up 42"),
            Ok(Instruction {
                dir: Direction::Vertical,
                value: -42
            })
        );
        assert_eq!(
            parse_instruction("down 9001"),
            Ok(Instruction {
                dir: Direction::Vertical,
                value: 9001
            })
        );
        assert_eq!(
            parse_instruction("left!"),
            Err("Unable to parse instruction 'left!'".to_owned())
        );
        assert_eq!(
            parse_instruction("forward abit"),
            Err("Unable to parse value of instruction 'forward abit': invalid digit found in string".to_owned())
        );
        assert_eq!(
            parse_instruction("backwards 3"),
            Err("Unknown direction: 'backwards'".to_owned())
        );
    }

    #[test]
    fn solve_puzzle_one_works_with_exapmple() {
        // given
        let instructions = parse_instructions(
            r"forward 5
down 5
forward 8
up 3
down 8
forward 2",
        )
        .expect("Expected valid instructions");

        // when
        let (x, y) = solve_puzzle_one(&instructions);

        // then
        assert_eq!(x, 15);
        assert_eq!(y, 10);
    }

    #[test]
    fn solve_puzzle_two_works_with_example() {
        // given
        let instructions = parse_instructions(
            r"forward 5
down 5
forward 8
up 3
down 8
forward 2",
        )
        .expect("Expected valid instructions");

        // when
        let (x, y) = solve_puzzle_two(&instructions);

        // then
        assert_eq!(x, 15);
        assert_eq!(y, 60);
    }
}
