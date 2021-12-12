use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let edges = parse_edges(&content)?;

    let all_paths = find_all_paths(&edges, vec![START_VERTICE], true);
    println!(
        "There are {} paths from start to end that visit small caves at most once",
        all_paths.len()
    );

    let all_paths_puzzle_2 = find_all_paths(&edges, vec![START_VERTICE], false);
    println!("There are {} paths from start to end that visit one small cave at most twice and other small caves at most once", all_paths_puzzle_2.len());

    Ok(())
}

const START_VERTICE: &str = "start";
const END_VERTICE: &str = "end";

fn find_all_paths<'a>(
    edges: &'a Edges,
    path: Vec<&'a str>,
    visited_small_cave_twice: bool,
) -> Vec<Vec<&'a str>> {
    let from: &str = path.last().unwrap_or(&START_VERTICE);
    if from == END_VERTICE {
        return vec![path];
    }
    if let Some(vertices) = edges.get(&from) {
        vertices
            .iter()
            .filter_map(|vertice| {
                if is_large_cave(vertice) {
                    Some((vertice, visited_small_cave_twice))
                } else {
                    let count = path.iter().filter(|v| v == &vertice).count();
                    if count == 0 {
                        Some((vertice, visited_small_cave_twice))
                    } else if !visited_small_cave_twice
                        && count == 1
                        && vertice != &START_VERTICE
                        && vertice != &END_VERTICE
                    {
                        Some((vertice, true))
                    } else {
                        None
                    }
                }
            })
            .flat_map(|(vertice, visited_twice)| {
                let mut p = path.clone();
                p.push(vertice);
                find_all_paths(edges, p, visited_twice)
            })
            .collect()
    } else {
        vec![]
    }
}

fn is_large_cave(vertice: &str) -> bool {
    vertice
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

type Edges<'a> = HashMap<&'a str, Vec<&'a str>>;

fn parse_edges(content: &str) -> Result<HashMap<&str, Vec<&str>>, String> {
    let mut edges: Edges = HashMap::with_capacity(32);
    for line in content.lines() {
        let (v1, v2) = line
            .split_once('-')
            .map(|(v1, v2)| (v1.trim(), v2.trim()))
            .ok_or_else(|| format!("Unable to parse vertice '{}'", line))?;
        edges
            .entry(v1)
            .or_insert_with(|| Vec::with_capacity(8))
            .push(v2);
        edges
            .entry(v2)
            .or_insert_with(|| Vec::with_capacity(8))
            .push(v1);
    }
    Ok(edges)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    const SIMPLE_EXAMPLE: &str = r"start-A
start-b
A-c
A-b
b-d
A-end
b-end
";

    #[test]
    fn find_all_paths_works_simple_example_puzzle_1() {
        // given
        let edges = parse_edges(SIMPLE_EXAMPLE).expect("Expected successful parsing");

        // when
        let paths = find_all_paths(&edges, vec![START_VERTICE], true);

        // then
        let path_set: HashSet<Vec<&str>> = paths.into_iter().collect();
        let expected_paths: HashSet<Vec<&str>> = [
            vec!["start", "A", "b", "A", "c", "A", "end"],
            vec!["start", "A", "b", "A", "end"],
            vec!["start", "A", "b", "end"],
            vec!["start", "A", "c", "A", "b", "A", "end"],
            vec!["start", "A", "c", "A", "b", "end"],
            vec!["start", "A", "c", "A", "end"],
            vec!["start", "A", "end"],
            vec!["start", "b", "A", "c", "A", "end"],
            vec!["start", "b", "A", "end"],
            vec!["start", "b", "end"],
        ]
        .into_iter()
        .collect();

        assert_eq!(path_set, expected_paths);
    }

    #[test]
    fn find_all_paths_works_for_simple_example_puzzle2() {
        // given
        let edges = parse_edges(SIMPLE_EXAMPLE).expect("Expected successful parsing");

        // when
        let paths = find_all_paths(&edges, vec![START_VERTICE], false);

        // then
        assert_eq!(paths.len(), 36);
    }
}
