use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let (template, rules) = parse(&content)?;

    let after_10_steps = grow_steps(template.clone(), &rules, 10);
    if let Some(score) = score(&after_10_steps) {
        println!("most common - least common: {}", score);
    } else {
        println!("I accidentally a polymer is this dangerous?");
    }

    if let Some(score_after_40_steps) = score_after_steps(&template, &rules, 40) {
        println!(
            "most common - least common after 40 steps: {}",
            score_after_40_steps
        );
    } else {
        println!("There is no polymer.");
    }

    Ok(())
}

fn score_after_steps(
    template: &[char],
    rules: &HashMap<(char, char), char>,
    n_steps: u64,
) -> Option<u64> {
    let mut letter_counts: HashMap<char, u64> =
        template
            .iter()
            .copied()
            .fold(HashMap::with_capacity(template.len()), |mut lc, c| {
                let counter = lc.entry(c).or_insert(0);
                *counter += 1;
                lc
            });
    let mut tuple_counts: HashMap<(char, char), u64> =
        template
            .windows(2)
            .fold(HashMap::with_capacity(rules.len()), |mut tc, tuple| {
                let counter = tc.entry((tuple[0], tuple[1])).or_insert(0);
                *counter += 1;
                tc
            });

    for _ in 0..n_steps {
        let mut next_tuple_counts: HashMap<(char, char), u64> =
            HashMap::with_capacity(tuple_counts.len() * 2);
        for (tuple, tuple_count) in tuple_counts {
            if let Some(inserted) = rules.get(&tuple).copied() {
                let letter_count = letter_counts.entry(inserted).or_insert(0);
                *letter_count += tuple_count;
                let first_tuple_count = next_tuple_counts.entry((tuple.0, inserted)).or_insert(0);
                *first_tuple_count += tuple_count;
                let second_tuple_count = next_tuple_counts.entry((inserted, tuple.1)).or_insert(0);
                *second_tuple_count += tuple_count;
            } else {
                next_tuple_counts.insert(tuple, tuple_count);
            }
        }
        tuple_counts = next_tuple_counts;
    }

    let min = letter_counts.values().min()?;
    let max = letter_counts.values().max()?;

    Some(max - min)
}

fn score(polymer: &[char]) -> Option<usize> {
    let counts: HashMap<char, usize> =
        polymer
            .iter()
            .copied()
            .fold(HashMap::with_capacity(26), |mut counts, c| {
                let counter = counts.entry(c).or_insert(0);
                *counter += 1;
                counts
            });
    let min = counts.values().min()?;
    let max = counts.values().max()?;

    Some(max - min)
}

fn grow_steps(
    template: Vec<char>,
    rules: &HashMap<(char, char), char>,
    n_steps: usize,
) -> Vec<char> {
    let mut polymer = template;
    for _ in 0..n_steps {
        polymer = grow_step(&polymer, rules);
    }
    polymer
}

fn grow_step(polymer: &[char], rules: &HashMap<(char, char), char>) -> Vec<char> {
    polymer
        .first()
        .iter()
        .copied()
        .copied()
        .chain(
            polymer
                .windows(2)
                .flat_map(|w| {
                    if let Some(inserted) = rules.get(&(w[0], w[1])) {
                        // this "array of options" stuff is here because I can't figure out a way to return two
                        // arrays of different lengths in a flat_map
                        [Some(*inserted), Some(w[1])]
                    } else {
                        [None, Some(w[1])]
                    }
                })
                .flatten(),
        )
        .collect()
}

type Input = (Vec<char>, HashMap<(char, char), char>);

fn parse(content: &str) -> Result<Input, String> {
    let mut lines = content.lines();
    let template: Vec<char> = lines
        .next()
        .ok_or_else(|| "Unable to find polymer template in input".to_owned())?
        .chars()
        .collect();

    if lines.next() != Some("") {
        return Err("Expected empty line after polymer template".to_owned());
    }

    let rules: HashMap<(char, char), char> = lines
        .map(parse_rule)
        .collect::<Result<HashMap<(char, char), char>, String>>()?;

    Ok((template, rules))
}

fn parse_rule(line: &str) -> Result<((char, char), char), String> {
    let (pair, inserted) = line
        .split_once(" -> ")
        .ok_or_else(|| format!("Unable to find separator in line '{}'", line))?;
    let mut pair_chars = pair.chars();
    let p1 = pair_chars.next().ok_or_else(|| {
        format!(
            "Unable to find first part of the left side in line '{}'",
            line
        )
    })?;
    let p2 = pair_chars.next().ok_or_else(|| {
        format!(
            "Unable to find second part of the left side in line '{}'",
            line
        )
    })?;
    let inserted = inserted
        .chars()
        .next()
        .ok_or_else(|| format!("Unable to find right side in line '{}'", line))?;

    Ok(((p1, p2), inserted))
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = r"NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C
";

    #[test]
    fn grow_steps_works_for_example() {
        // given
        let (template, rules) = parse(EXAMPLE_INPUT).expect("Expected successful parseing");

        // when
        let polymer = grow_steps(template, &rules, 4);

        // then
        assert_eq!(
            polymer,
            "NBBNBNBBCCNBCNCCNBBNBBNBBBNBBNBBCBHCBHHNHCBBCBHCB"
                .chars()
                .collect::<Vec<char>>()
        );
    }

    #[test]
    fn score_after_steps_works_for_example() {
        // given
        let (template, rules) = parse(EXAMPLE_INPUT).expect("Expected successful parseing");

        // when
        let score = score_after_steps(&template, &rules, 40);

        // then
        assert_eq!(score, Some(2188189693529));
    }
}
