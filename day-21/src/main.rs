use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let (startpos1, startpos2) = parse(&content)?;

    let result = simulate(startpos1, startpos2);
    println!(
        "The score of the losing player times the number of times the die was rolled: {}",
        result
    );

    Ok(())
}

fn simulate(start1: u64, start2: u64) -> u64 {
    // simulating this will probably bite me in the arse in part 2, but it's easy and I have no
    // idea in which direction to optimize for part 2, so let's just do it.
    let mut die_counter = 0;
    let mut pos1 = start1;
    let mut pos2 = start2;
    let mut points1 = 0;
    let mut points2 = 0;

    loop {
        pos1 = ((pos1 - 1) + die_counter * 3 + 6) % 10 + 1;
        points1 += pos1;
        die_counter += 3;
        if points1 >= 1000 {
            return points2 * die_counter;
        }
        pos2 = ((pos2 - 1) + die_counter * 3 + 6) % 10 + 1;
        points2 += pos2;
        die_counter += 3;
        if points2 >= 1000 {
            return points1 * die_counter;
        }
    }
}

fn parse(input: &str) -> Result<(u64, u64), String> {
    let (first, second) = input
        .split_once('\n')
        .ok_or_else(|| "not enough lines in input".to_owned())?;

    let pos1: u64 = first
        .strip_prefix("Player 1 starting position: ")
        .ok_or_else(|| format!("unexpected prefix in line: '{}'", first))?
        .trim()
        .parse()
        .map_err(|e| format!("unable to parse starting position for player 1: {}", e))?;
    if pos1 == 0 {
        return Err("player 1 starting position must be greater than 0".to_owned());
    }

    let pos2: u64 = second
        .strip_prefix("Player 2 starting position: ")
        .ok_or_else(|| format!("unexpected prefix in line: '{}'", first))?
        .trim()
        .parse()
        .map_err(|e| format!("unable to parse starting position for player 2: {}", e))?;

    if pos2 == 0 {
        return Err("player 2 starting position must greater than 0".to_owned());
    }

    Ok((pos1, pos2))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simulate_works_for_example() {
        // given
        let start1 = 4;
        let start2 = 8;

        // when
        let result = simulate(start1, start2);

        // then
        assert_eq!(result, 739785);
    }
}
