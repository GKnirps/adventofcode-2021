use std::collections::VecDeque;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let initial_map = parse(&content)?;

    let flashes_after_100 = run_steps(initial_map.clone(), 100);
    println!(
        "After 100 steps, there have been {} flashes in total",
        flashes_after_100
    );

    let steps_until_sync = run_until_sync(initial_map);
    println!(
        "After {} steps, all octopuses flash in sync",
        steps_until_sync
    );

    Ok(())
}

fn run_steps(mut octo_map: OctoMap, n_steps: u64) -> u64 {
    let mut flash_counter = 0;
    for _ in 0..n_steps {
        let (next_octo_map, flashes) = run_step(octo_map);
        octo_map = next_octo_map;
        flash_counter += flashes;
    }
    flash_counter
}

fn run_until_sync(mut octo_map: OctoMap) -> u64 {
    let mut counter = 0;
    let mut flashes = 0;
    while flashes < octo_map.energy.len() as u64 {
        let (next_octo_map, f) = run_step(octo_map);
        flashes = f;
        octo_map = next_octo_map;
        counter += 1;
    }
    counter
}

fn run_step(mut octo_map: OctoMap) -> (OctoMap, u64) {
    let mut queue: VecDeque<usize> = VecDeque::with_capacity(octo_map.energy.len());
    for (i, e) in octo_map.energy.iter_mut().enumerate() {
        *e += 1;
        if *e == 10 {
            queue.push_back(i);
        }
    }
    let mut flash_count = 0;
    while let Some(pos) = queue.pop_front() {
        flash_count += 1;
        for n in octo_map.moore_neighbours(pos) {
            octo_map.energy[n] += 1;
            if octo_map.energy[n] == 10 {
                queue.push_back(n);
            }
        }
    }

    for e in octo_map.energy.iter_mut() {
        if *e > 9 {
            *e = 0;
        }
    }

    (octo_map, flash_count)
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct OctoMap {
    edge_length: usize,
    energy: Vec<u8>,
}

// I know I don't really need an iterator for that but I want one!
struct MooreNeighbours {
    edge_length: isize,
    center: isize,
    neighbour_index: usize,
}

impl OctoMap {
    fn moore_neighbours(&self, pos: usize) -> MooreNeighbours {
        MooreNeighbours {
            edge_length: self.edge_length as isize,
            center: pos as isize,
            neighbour_index: 0,
        }
    }
}

const MOORE_NEIGHBOURS: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, -0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

impl Iterator for MooreNeighbours {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.neighbour_index < MOORE_NEIGHBOURS.len() {
            let (dx, dy) = MOORE_NEIGHBOURS[self.neighbour_index];
            self.neighbour_index += 1;
            let x = self.center % self.edge_length + dx;
            let y = self.center / self.edge_length + dy;
            if x < 0 || y < 0 || x >= self.edge_length || y >= self.edge_length {
                continue;
            } else {
                return Some((x + y * self.edge_length) as usize);
            }
        }
        None
    }

    // I _could_ also implement `size_hint` here, but I really don't need this one here
}

fn parse(content: &str) -> Result<OctoMap, String> {
    let energy: Vec<u8> = content
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect();
    if energy.len() != 100 {
        return Err(format!(
            "Expected exactly 100 dumbo octopuses, found {}",
            energy.len()
        ));
    }
    Ok(OctoMap {
        edge_length: 10,
        energy,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = r"5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526
";
    #[test]
    fn run_steps_works_for_example() {
        // given
        let before = parse(EXAMPLE_INPUT).expect("expected successful parsing");

        // when
        let flashes = run_steps(before, 100);

        // then
        assert_eq!(flashes, 1656);
    }

    #[test]
    fn run_until_sync_works_for_example() {
        // given
        let before = parse(EXAMPLE_INPUT).expect("expected successful parsing");

        // when
        let steps = run_until_sync(before);

        // then
        assert_eq!(steps, 195);
    }

    #[test]
    fn run_step_works_for_example_step_1() {
        // given
        let before = parse(
            r"6594254334
3856965822
6375667284
7252447257
7468496589
5278635756
3287952832
7993992245
5957959665
6394862637
",
        )
        .expect("expected successful parsing");
        let expected_after = parse(
            r"8807476555
5089087054
8597889608
8485769600
8700908800
6600088989
6800005943
0000007456
9000000876
8700006848
",
        )
        .expect("expected successful parsing");

        // when
        let (after, flashes) = run_step(before);

        // then
        assert_eq!(after, expected_after);
        assert_eq!(flashes, 35);
    }
}
