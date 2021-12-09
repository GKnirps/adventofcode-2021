use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let height_map = parse(&content)?;

    let risk = get_low_points_risk_level(&height_map);
    println!("Sum of risk levels on low points is {}", risk);

    Ok(())
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct HeightMap {
    width: usize,
    height: usize,
    values: Vec<u8>,
}

fn top(i: usize, height_map: &HeightMap) -> Option<(usize, u8)> {
    let width = height_map.width;
    if i >= width {
        Some((i - width, height_map.values[i - width]))
    } else {
        None
    }
}

fn bottom(i: usize, height_map: &HeightMap) -> Option<(usize, u8)> {
    height_map
        .values
        .get(i + height_map.width)
        .map(|v| (i + height_map.width, *v))
}

fn left(i: usize, height_map: &HeightMap) -> Option<(usize, u8)> {
    if i % height_map.width > 0 {
        Some((i - 1, height_map.values[i - 1]))
    } else {
        None
    }
}

fn right(i: usize, height_map: &HeightMap) -> Option<(usize, u8)> {
    if i % height_map.width < height_map.width - 1 {
        Some((i + 1, height_map.values[i + 1]))
    } else {
        None
    }
}

fn get_low_points_risk_level(height_map: &HeightMap) -> u32 {
    height_map
        .values
        .iter()
        .copied()
        .enumerate()
        .filter(|(i, value)| {
            let i = *i;
            let value = *value;
            [
                top(i, height_map),
                bottom(i, height_map),
                left(i, height_map),
                right(i, height_map),
            ]
            .into_iter()
            .flatten()
            .all(|(_, v)| v > value)
        })
        .map(|(_, v)| v as u32 + 1)
        .sum()
}

fn parse(input: &str) -> Result<HeightMap, String> {
    let values: Vec<u8> = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| {
            c.to_digit(10)
                .map(|d| d as u8)
                .ok_or_else(|| format!("unknown digit: {}", c))
        })
        .collect::<Result<Vec<u8>, String>>()?;
    // the step above should have failed for non-ascii values, so l.len() gives us the length we need
    let width = input
        .lines()
        .map(|l| l.len())
        .next()
        .ok_or_else(|| "expected at least one line".to_owned())?;
    if values.len() % width != 0 {
        return Err(format!(
            "total size {} is not dividable by width {}",
            values.len(),
            width
        ));
    }
    let height = values.len() / width;

    Ok(HeightMap {
        width,
        height,
        values,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_works_for_correct_input() {
        // given
        let input = r"123
456
";

        // when
        let result = parse(input);

        // then
        assert_eq!(
            result,
            Ok(HeightMap {
                width: 3,
                height: 2,
                values: vec![1, 2, 3, 4, 5, 6]
            })
        );
    }

    const EXAMPLE_INPUT: &str = r"2199943210
3987894921
9856789892
8767896789
9899965678
";

    #[test]
    fn get_low_points_risk_level_works_for_example() {
        // given
        let height_map = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let risk = get_low_points_risk_level(&height_map);

        // then
        assert_eq!(risk, 15);
    }

    #[test]
    fn get_low_points_risk_level_works_for_modified_example() {
        // given
        let height_map = parse(
            r"2199943210
3987894921
1856789892
8767896789
9899965678
",
        )
        .expect("Expected successful parsing");

        // when
        let risk = get_low_points_risk_level(&height_map);

        // then
        assert_eq!(risk, 17);
    }

    #[test]
    fn get_low_points_works_for_single_row() {
        // given
        let height_map = parse("19191\n").expect("Expected successful parsing");

        // when
        let risk = get_low_points_risk_level(&height_map);

        // then
        assert_eq!(risk, 6);
    }

    #[test]
    fn get_low_points_works_for_single_column() {
        // given
        let height_map = parse("1\n9\n1\n9\n1\n").expect("Expected successful parsing");

        // when
        let risk = get_low_points_risk_level(&height_map);

        // then
        assert_eq!(risk, 6);
    }

    #[test]
    fn get_low_points_works_for_edge_cases() {
        // given
        let height_map = parse(
            r"19191
99999
19191
99999
19191
",
        )
        .expect("Expected successful parsing");

        // when
        let risk = get_low_points_risk_level(&height_map);

        // then
        assert_eq!(risk, 18);
    }
}
