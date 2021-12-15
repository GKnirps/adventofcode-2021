use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
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

    let large_cavern = expand_cavern(&cavern, 5);
    if let Some(risk) = shortest_path(&large_cavern, 0, large_cavern.risk.len() - 1) {
        println!(
            "The least risky path through the large cavern has a risk value of {}",
            risk
        );
    } else {
        println!("There is no path, risky or otherwise. Are you sure you did not mess up your pathfinding alogorithm?");
    }

    Ok(())
}

fn shortest_path(cavern: &Cavern, start: usize, goal: usize) -> Option<u32> {
    let mut queue: BinaryHeap<VerticeDistance> = BinaryHeap::with_capacity(cavern.risk.len());
    queue.push(VerticeDistance {
        dist: 0,
        pos: start,
    });

    let mut visited: HashMap<usize, u32> = HashMap::with_capacity(cavern.risk.len());

    while let Some(VerticeDistance { dist: d, pos: v }) = queue.pop() {
        if v == goal {
            return Some(d);
        }
        if visited.contains_key(&v) {
            // since we cannot easily remove deprecated entries on the heap, we just skip them when
            // they are popped
            continue;
        }
        visited.insert(v, d);
        for neighbour in cavern.von_neumann_neighbours(v) {
            if !visited.contains_key(&neighbour) {
                let risk = d + cavern.risk[neighbour] as u32;
                // we can't really update the distance for a given position in the queue, but we
                // can just add all unvisited neighbours and later skip duplicates
                queue.push(VerticeDistance {
                    dist: risk,
                    pos: neighbour,
                });
            }
        }
    }
    None
}

// This is required to use BTreeSet as halfway efficient priority queue
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
struct VerticeDistance {
    dist: u32,
    pos: usize,
}

impl PartialOrd for VerticeDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// The main importance is that everything can be sorted by distance, everything else is secondary
impl Ord for VerticeDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.dist.cmp(&other.dist) {
            Ordering::Equal => self.pos.cmp(&other.pos),
            ne => ne,
        }
        .reverse() // reverse the ordering because BinaryHeap gives out max values first
    }
}

fn expand_cavern(cavern: &Cavern, factor: usize) -> Cavern {
    let source_height = cavern.risk.len() / cavern.width;
    let width = cavern.width * factor;
    let risk: Vec<u8> = (0..cavern.risk.len() * factor * factor)
        .map(|i| {
            let x = i % width;
            let y = i / width;
            let source_x = x % cavern.width;
            let source_y = y % source_height;
            let source_risk = cavern.risk[source_x + cavern.width * source_y];
            let risk = source_risk as usize + x / cavern.width + y / source_height;
            (if risk > 9 { risk % 10 + 1 } else { risk }) as u8
        })
        .collect();
    Cavern { width, risk }
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

    #[test]
    fn expand_cavern_wrap_around_works_for_example() {
        // given
        let cavern = Cavern {
            width: 1,
            risk: vec![8],
        };

        // when
        let larger = expand_cavern(&cavern, 5);

        // then
        assert_eq!(larger.width, 5);
        assert_eq!(
            larger.risk,
            &[8, 9, 1, 2, 3, 9, 1, 2, 3, 4, 1, 2, 3, 4, 5, 2, 3, 4, 5, 6, 3, 4, 5, 6, 7,]
        );
    }
}
