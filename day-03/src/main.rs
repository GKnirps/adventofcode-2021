use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let (numbers, relevant_bits) = parse(&content)?;

    let (gamma, epsilon) = solve_puzzle_one(&numbers, relevant_bits);

    println!(
        "gamma: {}, epsilon: {}, power consumption: {}",
        gamma,
        epsilon,
        gamma as u32 * epsilon as u32
    );

    Ok(())
}

const BITLEN: usize = u16::BITS as usize;
fn solve_puzzle_one(numbers: &[u16], relevant_bits: u16) -> (u16, u16) {
    let mut bit_counter: [usize; BITLEN] = [0; BITLEN];
    for number in numbers {
        for i in 0..BITLEN {
            if (number >> i) & 1 == 1 {
                bit_counter[BITLEN - 1 - i] += 1
            }
        }
    }
    let gamma: u16 = bit_counter.iter().fold(0, |num, count| {
        let shifted = num << 1;
        if *count > numbers.len() / 2 {
            shifted | 1
        } else {
            shifted
        }
    });
    (gamma, (!gamma) & relevant_bits)
}

fn parse(input: &str) -> Result<(Vec<u16>, u16), String> {
    let numbers = input
        .lines()
        .map(|l| u16::from_str_radix(l, 2).map_err(|e| format!("Unable to parse line '{}'", e)))
        .collect::<Result<Vec<u16>, String>>()?;
    let max_length = input.lines().map(|l| l.len()).max().unwrap_or(0);

    if max_length > BITLEN {
        return Err(format!(
            "Expected binary strings no longer than 16 bit, got length {}",
            max_length
        ));
    }

    let relevant_bits = u16::MAX >> (BITLEN - max_length);

    Ok((numbers, relevant_bits))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_works_correctly() {
        // given
        let input = r"00100
11110
00010
01010";

        // when
        let result = parse(input);

        // then
        assert_eq!(
            result,
            Ok((vec![0b00100, 0b11110, 0b00010, 0b01010], 0b11111))
        );
    }

    #[test]
    fn solve_puzzle_one_works_for_example() {
        // given
        let (numbers, relevant_bits) = parse(
            r"00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010",
        )
        .expect("Expected successful parsing");

        // when
        let (gamma, epsilon) = solve_puzzle_one(&numbers, relevant_bits);

        // then
        assert_eq!(gamma, 0b10110);
        assert_eq!(epsilon, 0b1001);
    }
}
