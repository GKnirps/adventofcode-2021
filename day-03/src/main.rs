use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let (numbers, bit_length) = parse(&content)?;

    let (gamma, epsilon) = solve_puzzle_one(&numbers, bit_length);

    println!(
        "gamma: {}, epsilon: {}, power consumption: {}",
        gamma,
        epsilon,
        gamma as u32 * epsilon as u32
    );

    let (oxygen, co2) = solve_puzzle_two(numbers, bit_length)?;

    println!(
        "O₂ generator rating: {}, CO₂ scrubber rating: {}, product: {}",
        oxygen,
        co2,
        oxygen as u32 * co2 as u32
    );

    Ok(())
}

const BITLEN: usize = u16::BITS as usize;
fn solve_puzzle_one(numbers: &[u16], bit_length: usize) -> (u16, u16) {
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
    let relevant_bits = u16::MAX >> (BITLEN - bit_length);
    (gamma, (!gamma) & relevant_bits)
}

fn solve_puzzle_two(numbers: Vec<u16>, bit_length: usize) -> Result<(u16, u16), String> {
    let mut oxygen: Vec<u16> = numbers.clone();
    for i in 0..bit_length {
        let bit_count: usize = oxygen
            .iter()
            .map(|n| ((n >> (bit_length - 1 - i)) & 1) as usize)
            .sum();
        let most_common: u16 = if bit_count * 2 >= oxygen.len() { 1 } else { 0 };
        oxygen = oxygen
            .into_iter()
            .filter(|n| (n >> (bit_length - 1 - i)) & 1 == most_common)
            .collect();
    }
    if oxygen.len() != 1 {
        return Err(format!(
            "did not find exactly one oxygen generator rating, found {} instead",
            oxygen.len()
        ));
    }

    // yeah, copy & past, I know.
    let mut co2: Vec<u16> = numbers;
    for i in 0..bit_length {
        let bit_count: usize = co2
            .iter()
            .map(|n| ((n >> (bit_length - 1 - i)) & 1) as usize)
            .sum();
        let least_common: u16 = if bit_count * 2 >= co2.len() { 0 } else { 1 };
        co2 = co2
            .into_iter()
            .filter(|n| (n >> (bit_length - 1 - i)) & 1 == least_common)
            .collect();
        if co2.len() == 1 {
            break;
        }
    }
    if co2.len() != 1 {
        return Err(format!(
            "did not find exactly one CO₂ scrubber rating, found {} instead",
            co2.len()
        ));
    }

    Ok((oxygen[0], co2[0]))
}

fn parse(input: &str) -> Result<(Vec<u16>, usize), String> {
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

    Ok((numbers, max_length))
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
        assert_eq!(result, Ok((vec![0b00100, 0b11110, 0b00010, 0b01010], 5)));
    }

    #[test]
    fn solve_puzzle_one_works_for_example() {
        // given
        let (numbers, bit_length) = parse(
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
        let (gamma, epsilon) = solve_puzzle_one(&numbers, bit_length);

        // then
        assert_eq!(gamma, 0b10110);
        assert_eq!(epsilon, 0b1001);
    }

    #[test]
    fn solve_puzzle_two_works_for_example() {
        // given
        let (numbers, bit_length) = parse(
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
        let result = solve_puzzle_two(numbers, bit_length);

        // then
        assert_eq!(result, Ok((23, 10)))
    }
}
