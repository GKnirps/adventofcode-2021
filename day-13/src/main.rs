use std::cmp::Ordering;
use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let (initial_dots, folding_instructions) = parse(&content)?;

    // puzzle 1
    if let Some(first_fold) = folding_instructions.get(0..1) {
        let dots = fold_dots(initial_dots.clone(), first_fold);
        println!(
            "After the first fold, there are {} distinct dots",
            dots.len()
        );
    }

    // puzzle 2
    let final_dots = fold_dots(initial_dots, &folding_instructions);
    print_dots(&final_dots);

    Ok(())
}

fn fold_dots(mut dots: HashSet<Dot>, instructions: &[Fold]) -> HashSet<Dot> {
    for fold in instructions {
        let fold_fn = match fold.orientation {
            Orientation::Horizontal => fold_vertical,
            Orientation::Vertical => fold_horizontal,
        };
        // allocating a new set each time? Makes it simpler, and we don't fold _that_ often
        dots = dots
            .into_iter()
            .filter_map(|dot| fold_fn(dot, fold.position))
            .collect();
    }
    dots
}

fn fold_vertical((x, y): Dot, pos: i64) -> Option<Dot> {
    match y.cmp(&pos) {
        Ordering::Less => Some((x, y)),
        Ordering::Greater => Some((x, -y + pos * 2)),
        Ordering::Equal => None,
    }
}

fn fold_horizontal((x, y): Dot, pos: i64) -> Option<Dot> {
    match x.cmp(&pos) {
        Ordering::Less => Some((x, y)),
        Ordering::Greater => Some((-x + pos * 2, y)),
        Ordering::Equal => None,
    }
}

fn print_dots(dots: &HashSet<Dot>) {
    if dots.is_empty() {
        return;
    }
    let min_x: i64 = dots.iter().map(|(x, _)| x).copied().min().unwrap();
    let min_y: i64 = dots.iter().map(|(_, y)| y).copied().min().unwrap();
    let max_x: i64 = dots.iter().map(|(x, _)| x).copied().max().unwrap();
    let max_y: i64 = dots.iter().map(|(_, y)| y).copied().max().unwrap();

    let width: usize = (max_x - min_x) as usize + 1;
    let height: usize = (max_y - min_y) as usize + 1;
    let mut dot_map: Vec<bool> = vec![false; width * height];

    for (x, y) in dots {
        dot_map[(x - min_x) as usize + (y - min_y) as usize * width] = true;
    }

    // this won't print fast, but this is no bottleneck, so who cares?
    for row in dot_map.chunks(width) {
        for col in row {
            if *col {
                print!("█");
            } else {
                print!(" ");
            }
        }
        println!();
    }
}

fn parse(content: &str) -> Result<(HashSet<Dot>, Vec<Fold>), String> {
    let (dot_content, fold_content) = content.split_once("\n\n").ok_or_else(|| {
        "Unable to find separator between dots and folding instructions".to_owned()
    })?;

    let dots = dot_content
        .lines()
        .map(parse_dot)
        .collect::<Result<HashSet<Dot>, String>>()?;
    let folds = fold_content
        .lines()
        .map(parse_fold)
        .collect::<Result<Vec<Fold>, String>>()?;
    Ok((dots, folds))
}

type Dot = (i64, i64);

fn parse_dot(line: &str) -> Result<Dot, String> {
    let (x, y) = line
        .split_once(',')
        .ok_or_else(|| format!("unable to parse dot '{}'", line))?;
    let x = x
        .parse::<i64>()
        .map_err(|e| format!("unable to parse x of dot '{}': {}", line, e))?;
    let y = y
        .parse::<i64>()
        .map_err(|e| format!("unable to parse y of dot '{}': {}", line, e))?;
    Ok((x, y))
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Fold {
    orientation: Orientation,
    position: i64,
}

fn parse_fold(line: &str) -> Result<Fold, String> {
    if let Some(s) = line.strip_prefix("fold along x=") {
        let position = s
            .parse::<i64>()
            .map_err(|e| format!("unable to parse position for fold '{}': {}", line, e))?;
        Ok(Fold {
            orientation: Orientation::Vertical,
            position,
        })
    } else if let Some(s) = line.strip_prefix("fold along y=") {
        let position = s
            .parse::<i64>()
            .map_err(|e| format!("unable to parse position for fold '{}': {}", line, e))?;
        Ok(Fold {
            orientation: Orientation::Horizontal,
            position,
        })
    } else {
        Err(format!("unable to parse fold '{}'", line))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = r"6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5
";

    #[test]
    fn fold_dots_works_for_example() {
        // given
        let (dots, instructions) = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let result = fold_dots(dots, &instructions);

        // then
        print_dots(&result);
        assert_eq!(result.len(), 16);
    }
}
