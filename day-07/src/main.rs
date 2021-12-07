use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let initial_positions = parse(&content)?;

    let alignment_fuel = brute_force_alignment(&initial_positions);
    println!(
        "The minimal total fuel usage for aligning the crabsubs is {}",
        alignment_fuel
    );

    Ok(())
}

fn brute_force_alignment(positions: &[u32]) -> u32 {
    let min: u32 = positions.iter().min().copied().unwrap_or(0);
    let max: u32 = positions.iter().max().copied().unwrap_or(0);

    (min..=max)
        .map(|align_pos| {
            positions
                .iter()
                .map(|pos| {
                    if *pos < align_pos {
                        align_pos - pos
                    } else {
                        pos - align_pos
                    }
                })
                .sum()
        })
        .min()
        .unwrap_or(0)
}

fn parse(input: &str) -> Result<Vec<u32>, String> {
    input
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<u32>()
                .map_err(|e| format!("Unable to parse position: {}", e))
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn brute_force_alignment_works_for_example() {
        // given
        let positions = parse("16,1,2,0,4,2,7,1,2,14\n").expect("Expected successful parsing");

        // when
        let fuel = brute_force_alignment(&positions);

        // then
        assert_eq!(fuel, 37);
    }
}
