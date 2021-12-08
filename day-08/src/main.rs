use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let input = parse(&content)?;

    let trivial_values = count_trivial_values(&input);
    println!(
        "There are {} values that are trivial to determine",
        trivial_values
    );

    if let Some(number) = sum_numbers(&input) {
        println!("Sum of numbers is {}", number);
    } else {
        println!("Unable to find correct patterns for all values");
    }

    Ok(())
}

type Patterns = [u8; 10];
type Values = [u8; 4];

fn count_trivial_values(input: &[(Patterns, Values)]) -> usize {
    input
        .iter()
        .flat_map(|(_, values)| values)
        .filter(|v| {
            let counter = count_1_bits(**v);
            counter == 2 || counter == 3 || counter == 4 || counter == 7
        })
        .count()
}

fn count_1_bits(mut value: u8) -> u8 {
    let mut counter: u8 = 0;
    while value != 0 {
        counter += value & 1;
        value >>= 1;
    }
    counter
}

fn sum_numbers(input: &[(Patterns, Values)]) -> Option<u32> {
    input
        .iter()
        .map(|(patterns, values)| determine_number(patterns, values))
        .fold(Some(0), |sum, n| Some(sum? + n?))
}

fn determine_number(patterns: &Patterns, values: &Values) -> Option<u32> {
    let pattern_1 = patterns.iter().copied().find(|v| count_1_bits(*v) == 2)?;
    let pattern_7 = patterns.iter().copied().find(|v| count_1_bits(*v) == 3)?;
    let pattern_4 = patterns.iter().copied().find(|v| count_1_bits(*v) == 4)?;
    let pattern_8 = patterns.iter().copied().find(|v| count_1_bits(*v) == 7)?;
    let pattern_3 = patterns
        .iter()
        .copied()
        .find(|v| count_1_bits(*v) == 5 && (*v & pattern_1) == pattern_1)?;
    let pattern_2 = patterns
        .iter()
        .copied()
        .find(|v| count_1_bits(*v) == 5 && count_1_bits(*v & pattern_4) == 2)?;
    let pattern_5 = patterns
        .iter()
        .copied()
        .find(|v| *v != pattern_3 && count_1_bits(*v) == 5 && count_1_bits(*v & pattern_4) == 3)?;
    let pattern_6 = patterns
        .iter()
        .copied()
        .find(|v| count_1_bits(*v) == 6 && count_1_bits(*v & pattern_7) == 2)?;
    let pattern_9 = patterns
        .iter()
        .copied()
        .find(|v| count_1_bits(*v) == 6 && (*v & pattern_4) == pattern_4)?;
    let pattern_0 = patterns
        .iter()
        .copied()
        .find(|v| count_1_bits(*v) == 6 && *v != pattern_9 && (*v & pattern_1 == pattern_1))?;

    let ordered_patterns = [
        pattern_0, pattern_1, pattern_2, pattern_3, pattern_4, pattern_5, pattern_6, pattern_7,
        pattern_8, pattern_9,
    ];

    let mut result: u32 = 0;
    for value in values {
        result *= 10;
        result += ordered_patterns
            .iter()
            .copied()
            .enumerate()
            .find(|(_, p)| p == value)?
            .0 as u32;
    }
    Some(result)
}

fn parse(input: &str) -> Result<Vec<(Patterns, Values)>, String> {
    input.lines().map(parse_line).collect()
}

fn parse_line(line: &str) -> Result<(Patterns, Values), String> {
    let (left, right) = line
        .split_once(" | ")
        .ok_or_else(|| format!("No delimiter in line '{}'", line))?;
    let mut patterns: Patterns = [0; 10];
    for (i, code) in left.split_whitespace().enumerate() {
        if i >= patterns.len() {
            return Err(format!("found more than 8 patterns in line '{}'", line));
        }
        patterns[i] = parse_code(code)?;
    }
    let mut values: Values = [0; 4];
    for (i, code) in right.split_whitespace().enumerate() {
        if i >= values.len() {
            return Err(format!("found more than 4 values in line '{}'", line));
        }
        values[i] = parse_code(code)?;
    }
    Ok((patterns, values))
}

fn parse_code(code: &str) -> Result<u8, String> {
    code.chars()
        .map(|c| match c {
            'a' => Ok(0b0000001),
            'b' => Ok(0b0000010),
            'c' => Ok(0b0000100),
            'd' => Ok(0b0001000),
            'e' => Ok(0b0010000),
            'f' => Ok(0b0100000),
            'g' => Ok(0b1000000),
            other => Err(format!("Unknown character: '{}'", other)),
        })
        .try_fold(0u8, |a, b| Ok(a | b?))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_line_works_for_example() {
        // given
        let input = r"be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe";

        // when
        let result = parse_line(input);

        // then
        assert_eq!(
            result,
            Ok((
                [
                    0b0010010, 0b1111111, 0b1111110, 0b1111101, 0b1010110, 0b1111100, 0b1111011,
                    0b0111110, 0b0101111, 0b0011010
                ],
                [0b1111111, 0b0111110, 0b1111110, 0b1010110]
            )),
        );
    }

    const EXAMPLE_INPUT: &str = r"be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce
";

    #[test]
    fn count_trivial_values_works_for_example() {
        // given
        let input = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let count = count_trivial_values(&input);

        // then
        assert_eq!(count, 26);
    }

    #[test]
    fn determine_number_works_for_example() {
        // given
        let (patterns, values) = parse_line(
            "acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab | cdfeb fcadb cdfeb cdbaf",
        )
        .expect("expected successful parsing");

        // when
        let number = determine_number(&patterns, &values);

        // then
        assert_eq!(number, Some(5353));
    }

    #[test]
    fn sum_numbers_works_for_example() {
        // given
        let input = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let sum = sum_numbers(&input);

        // then
        assert_eq!(sum, Some(61229));
    }
}
