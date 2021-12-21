use std::collections::HashMap;
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

    let (wins1, wins2) = winning_universes(startpos1, startpos2);
    println!(
        "Player 1 wins in {} universes, player 2 wins in {} universes",
        wins1, wins2
    );

    Ok(())
}

// for three dices rolled, pairs of (sum of eyes, number of combinations with that sum)
const THREE_ROLL_OUTCOMES: [(u64, u64); 7] =
    [(3, 1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)];

fn winning_universes(start1: u64, start2: u64) -> (u64, u64) {
    let mut wins1: u64 = 0;
    let mut wins2: u64 = 0;
    let mut ways1_by_pos_points: HashMap<(u64, u64), u64> = HashMap::with_capacity(1);
    ways1_by_pos_points.insert((start1, 0), 1);
    let mut ways2_by_pos_points: HashMap<(u64, u64), u64> = HashMap::with_capacity(1);
    ways2_by_pos_points.insert((start2, 0), 1);
    while !ways1_by_pos_points.is_empty() && !ways2_by_pos_points.is_empty() {
        let mut next_ways1_by_pos_points: HashMap<(u64, u64), u64> =
            HashMap::with_capacity(ways1_by_pos_points.len() * THREE_ROLL_OUTCOMES.len());
        let mut next_ways2_by_pos_points: HashMap<(u64, u64), u64> =
            HashMap::with_capacity(ways2_by_pos_points.len() * THREE_ROLL_OUTCOMES.len());
        let ways2: u64 = ways2_by_pos_points.iter().map(|(_, ways)| *ways).sum();
        for ((pos, points), ways) in ways1_by_pos_points {
            for (roll, roll_count) in THREE_ROLL_OUTCOMES {
                let next_pos = ((pos - 1) + roll) % 10 + 1;
                let next_points = points + next_pos;
                if next_points >= 21 {
                    wins1 += ways * roll_count * ways2;
                } else {
                    let next_ways: &mut u64 = next_ways1_by_pos_points
                        .entry((next_pos, next_points))
                        .or_insert(0);
                    *next_ways += ways * roll_count;
                }
            }
        }
        ways1_by_pos_points = next_ways1_by_pos_points;
        let ways1: u64 = ways1_by_pos_points.iter().map(|(_, ways)| *ways).sum();
        for ((pos, points), ways) in ways2_by_pos_points {
            for (roll, roll_count) in THREE_ROLL_OUTCOMES {
                let next_pos = ((pos - 1) + roll) % 10 + 1;
                let next_points = points + next_pos;
                if next_points >= 21 {
                    wins2 += ways * roll_count * ways1;
                } else {
                    let next_ways: &mut u64 = next_ways2_by_pos_points
                        .entry((next_pos, next_points))
                        .or_insert(0);
                    *next_ways += ways * roll_count;
                }
            }
        }
        ways2_by_pos_points = next_ways2_by_pos_points;
    }
    (wins1, wins2)
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

    #[test]
    fn winning_universes_works_for_example() {
        // given
        let start1 = 4;
        let start2 = 8;

        // when
        let wins = winning_universes(start1, start2);

        // then
        assert_eq!(wins, (444_356_092_776_315, 341_960_390_180_808));
    }
}
