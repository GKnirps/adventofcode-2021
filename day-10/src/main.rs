use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.lines().collect();

    let score = corruption_score(&lines);
    println!("The corruption score is {}", score);

    let completion = completion_score(&lines);
    println!("The completion score is {}", completion);

    Ok(())
}

fn corruption_score(lines: &[&str]) -> u32 {
    lines.iter().filter_map(|line| verify(*line).err()).sum()
}

fn completion_score(lines: &[&str]) -> u64 {
    let mut scores: Vec<u64> = lines.iter().filter_map(|line| verify(*line).ok()).collect();
    scores.sort_unstable();
    scores.get(scores.len() / 2).copied().unwrap_or(0)
}

fn verify(line: &str) -> Result<u64, u32> {
    let mut stack: Vec<char> = Vec::with_capacity(line.len());
    for c in line.chars() {
        match c {
            '[' | '(' | '{' | '<' => stack.push(c),
            ')' => match stack.pop() {
                Some('(') => (),
                _ => {
                    return Err(3);
                }
            },
            ']' => match stack.pop() {
                Some('[') => (),
                _ => {
                    return Err(57);
                }
            },
            '}' => match stack.pop() {
                Some('{') => (),
                _ => {
                    return Err(1197);
                }
            },
            '>' => match stack.pop() {
                Some('<') => (),
                _ => {
                    return Err(25137);
                }
            },
            _ => (),
        }
    }
    Ok(stack.iter().rev().fold(0, |score, c| {
        score * 5
            + match c {
                '(' => 1,
                '[' => 2,
                '{' => 3,
                '<' => 4,
                _ => 0,
            }
    }))
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = r"[({(<(())[]>[[{[]{<()<>>
[(()[<>])]({[<{<<[]>>(
{([(<{}[<>[]}>{[]{[(<()>
(((({<>}<{<{<>}{[]{[]{}
[[<[([]))<([[{}[[()]]]
[{[{({}]{}}([{[{{{}}([]
{<[[]]>}<{[{[{[]{()[[[]
[<(<(<(<{}))><([]([]()
<{([([[(<>()){}]>(<<{{
<{([{{}}[<[[[<>{}]]]>[]]
";

    #[test]
    fn corruption_score_works_for_example() {
        // given
        let lines: Vec<&str> = EXAMPLE_INPUT.lines().collect();

        // when
        let score = corruption_score(&lines);

        // then
        assert_eq!(score, 26397);
    }

    #[test]
    fn completion_score_works_for_example() {
        // given
        let lines: Vec<&str> = EXAMPLE_INPUT.lines().collect();

        // when
        let score = completion_score(&lines);

        // then
        assert_eq!(score, 288957);
    }
}
