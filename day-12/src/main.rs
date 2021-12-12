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

    let all_paths = find_all_paths(&edges, vec![START_VERTICE]);
    println!(
        "There are {} paths from start to end that visit small caves at most once",
        all_paths.len()
    );

    Ok(())
}

const START_VERTICE: &str = "start";
const END_VERTICE: &str = "end";

fn find_all_paths<'a>(edges: &'a Edges, path: Vec<&'a str>) -> Vec<Vec<&'a str>> {
    let from: &str = path.last().unwrap_or(&START_VERTICE);
    if from == END_VERTICE {
        return vec![path];
    }
    if let Some(vertices) = edges.get(&from) {
        vertices
            .iter()
            .filter(|vertice| is_large_cave(vertice) || !path.contains(vertice))
            .flat_map(|vertice| {
                let mut p = path.clone();
                p.push(vertice);
                find_all_paths(edges, p)
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

    #[test]
    fn find_all_paths_works_work_simple_example() {
        // given
        let edges = parse_edges(
            r"start-A
start-b
A-c
A-b
b-d
A-end
b-end
",
        )
        .expect("Expected successful parsing");

        // when
        let paths = find_all_paths(&edges, vec![START_VERTICE]);

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
}
