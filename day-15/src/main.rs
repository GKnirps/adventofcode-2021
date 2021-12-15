use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let cavern = parse(&content)?;

    if let Some(risk) = shortest_path(&cavern, 0, cavern.risk.len() - 1) {
        println!("The least risky path has a risk value of {}", risk);
    } else {
        println!("There is no path, risky or otherwise. Are you sure you did not mess up your pathfinding alogorithm?");
    }

    Ok(())
}

fn shortest_path(cavern: &Cavern, start: usize, goal: usize) -> Option<u32> {
    // this should be some kind of priority queue, but I don't want to implement one here
    // and std::collections::BinaryHeap is not easy to update, so I take this performance hit into
    // account and just iterate over all values when I want to get the lowest one
    let mut queue: HashMap<usize, u32> = HashMap::with_capacity(cavern.risk.len());
    queue.insert(start, 0);

    let mut visited: HashMap<usize, u32> = HashMap::with_capacity(cavern.risk.len());

    while let Some((v, d)) = queue.iter().map(|(v, d)| (*v, *d)).min_by_key(|(_, d)| *d) {
        if v == goal {
            return Some(d);
        }
        queue.remove(&v);
        visited.insert(v, d);
        for neighbour in cavern.von_neumann_neighbours(v) {
            if !visited.contains_key(&neighbour) {
                let risk = d + cavern.risk[neighbour] as u32;
                let queue_risk = queue.entry(neighbour).or_insert(risk);
                if *queue_risk > risk {
                    *queue_risk = risk;
                }
            }
        }
    }
    None
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Cavern {
    width: usize,
    risk: Vec<u8>,
}

fn parse(input: &str) -> Result<Cavern, String> {
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "Unable to find first line of cavern map".to_owned())?
        .chars()
        .count();
    let risk: Vec<u8> = input
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();

    if risk.len() % width != 0 {
        Err(format!(
            "The cavern does not seem to be a rectangle: area: {}, width: {}",
            risk.len(),
            width
        ))
    } else {
        Ok(Cavern { width, risk })
    }
}
// I know I don't really need an iterator for that but I want one!
struct VonNeumannNeighbours {
    width: isize,
    center: isize,
    neighbour_index: usize,
}

impl Cavern {
    fn von_neumann_neighbours(&self, pos: usize) -> VonNeumannNeighbours {
        VonNeumannNeighbours {
            width: self.width as isize,
            center: pos as isize,
            neighbour_index: 0,
        }
    }
}

const VON_NEUMANN_NEIGHBOURS: [(isize, isize); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

impl Iterator for VonNeumannNeighbours {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.neighbour_index < VON_NEUMANN_NEIGHBOURS.len() {
            let (dx, dy) = VON_NEUMANN_NEIGHBOURS[self.neighbour_index];
            self.neighbour_index += 1;
            let x = self.center % self.width + dx;
            let y = self.center / self.width + dy;
            if x < 0 || y < 0 || x >= self.width || y >= self.width {
                continue;
            } else {
                return Some((x + y * self.width) as usize);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = r"1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581
";

    #[test]
    fn shortest_path_works_for_example() {
        // given
        let cavern = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let least_risk = shortest_path(&cavern, 0, cavern.risk.len() - 1);

        // then
        assert_eq!(least_risk, Some(40));
    }
}
