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

    Ok(())
}

fn corruption_score(lines: &[&str]) -> u32 {
    lines.iter().copied().filter_map(first_corruption).sum()
}

fn first_corruption(line: &str) -> Option<u32> {
    let mut stack: Vec<char> = Vec::with_capacity(line.len());
    for c in line.chars() {
        match c {
            '[' | '(' | '{' | '<' => stack.push(c),
            ')' => match stack.pop() {
                Some('(') => (),
                _ => {
                    return Some(3);
                }
            },
            ']' => match stack.pop() {
                Some('[') => (),
                _ => {
                    return Some(57);
                }
            },
            '}' => match stack.pop() {
                Some('{') => (),
                _ => {
                    return Some(1197);
                }
            },
            '>' => match stack.pop() {
                Some('<') => (),
                _ => {
                    return Some(25137);
                }
            },
            _ => (),
        }
    }
    // line may be incomplete, but we may ignore those lines for now
    None
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
}
