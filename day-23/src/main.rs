use std::cmp;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let initial_pos = parse(&content)?;
    let small_burrow = create_small_burrow(&initial_pos);

    if let Some(least_energy) = find_least_energy(&small_burrow) {
        println!("The least energy to sort all amphipods is {}", least_energy);
    } else {
        println!("There is no way to get these amphipods to the right rooms.");
    }

    let large_burrow = create_large_burrow(&initial_pos);
    if let Some(least_energy) = find_least_energy(&large_burrow) {
        println!(
            "The least energy to sort all amphipods in the larger burrow is {}",
            least_energy
        );
    } else {
        println!("There is no way to sort this large burrow.");
    }

    Ok(())
}

// so this is basically a shortest-path problems, but each vertice on the path is a constellation of
// amphipod positions
fn find_least_energy<const SIZE: usize, const N_AMPHIPODS: usize>(
    burrow: &Burrow<SIZE, N_AMPHIPODS>,
) -> Option<u32> {
    let mut queue: BinaryHeap<StateCost<[Option<usize>; SIZE]>> = BinaryHeap::with_capacity(128);
    queue.push(StateCost {
        state: burrow.initial_state,
        cost: 0,
    });
    let mut seen: HashSet<[Option<usize>; SIZE]> = HashSet::with_capacity(256);

    while let Some(StateCost { state, cost }) = queue.pop() {
        if !seen.insert(state) {
            // already seen, do not need to look at again
            continue;
        }
        if is_final_state(burrow, &state) {
            return Some(cost);
        }
        add_reachable_states(&mut queue, burrow, state, cost);
    }
    None
}

fn add_reachable_states<const SIZE: usize, const N_AMPHIPODS: usize>(
    queue: &mut BinaryHeap<StateCost<[Option<usize>; SIZE]>>,
    burrow: &Burrow<SIZE, N_AMPHIPODS>,
    state: [Option<usize>; SIZE],
    cost: u32,
) {
    for (pos, amph_index) in state
        .iter()
        .enumerate()
        .filter_map(|(i, content)| Some((i, (*content)?)))
    {
        // if the amphipod is already in the room where it wants to be (and no other amphipod types
        // need to get out), we don't need to look at that amphipod anymore
        if at_rest_in_target_room(burrow, &state, pos, amph_index) {
            continue;
        }

        for (new_pos, new_pos_cost) in explore_reachable_positions(burrow, &state, pos, amph_index)
        {
            let mut new_state = state;
            new_state[pos] = None;
            new_state[new_pos] = Some(amph_index);
            queue.push(StateCost {
                cost: cost + new_pos_cost,
                state: new_state,
            });
        }
    }
}

// returns true if the given position is a target position for amph_index and only the correct
// amphipods are deeper in the room
fn at_rest_in_target_room<const SIZE: usize, const N_AMPHIPODS: usize>(
    burrow: &Burrow<SIZE, N_AMPHIPODS>,
    state: &[Option<usize>; SIZE],
    pos: usize,
    amph_index: usize,
) -> bool {
    if let Some(index) = burrow.target_positions[amph_index]
        .iter()
        .enumerate()
        .find(|(_, target_room_pos)| pos == **target_room_pos)
        .map(|(target_room_index, _)| target_room_index)
    {
        burrow.target_positions[amph_index]
            .get((index + 1)..)
            .unwrap_or(&[])
            .iter()
            .copied()
            .all(|position| state[position] == Some(amph_index))
    } else {
        false
    }
}

fn explore_reachable_positions<const SIZE: usize, const N_AMPHIPODS: usize>(
    burrow: &Burrow<SIZE, N_AMPHIPODS>,
    state: &[Option<usize>; SIZE],
    pos: usize,
    amph_index: usize,
) -> Vec<(usize, u32)> {
    let mut queue: VecDeque<(usize, u32)> = VecDeque::with_capacity(128);
    let mut seen: [bool; SIZE] = [false; SIZE];
    queue.push_back((pos, 0));

    let mut reachable: Vec<(usize, u32)> = Vec::with_capacity(SIZE);

    let movement_cost = MOVE_COSTS[amph_index];

    // we don't have uniform costs, but the room layout is a tree, so we still get an optimal
    // solution with a BFS
    while let Some((current, cost)) = queue.pop_front() {
        if seen[current] {
            continue;
        }
        seen[current] = true;

        // pos > 7 means the amphipod started in a room (and will move anywhere)
        // the other stuff means either the current room is the inner target room (we can/should move
        // there) or it is the outer target room and there is already the right amphipod in the
        // inner target room)
        // correct kind of amphipods
        if (pos >= 7 && current < 7) || at_rest_in_target_room(burrow, state, current, amph_index) {
            reachable.push((current, cost));
        }

        for (neighbour, cost_factor) in &burrow.layout[current] {
            if seen[*neighbour] {
                continue;
            }
            if state[*neighbour].is_some() {
                // space occupied, can't move here
                continue;
            }
            let neighbour_cost = cost + cost_factor * movement_cost;
            queue.push_back((*neighbour, neighbour_cost))
        }
    }
    reachable
}

fn is_final_state<const SIZE: usize, const N_AMPHIPODS: usize>(
    burrow: &Burrow<SIZE, N_AMPHIPODS>,
    state: &[Option<usize>; SIZE],
) -> bool {
    burrow
        .target_positions
        .iter()
        .enumerate()
        .all(|(amphipod_id, desired_positions)| {
            desired_positions
                .iter()
                .copied()
                .all(|pos| state[pos] == Some(amphipod_id))
        })
}

// struct with custom cmp operator for the priority queue
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
struct StateCost<T> {
    cost: u32,
    state: T,
}

impl<T: PartialOrd + Ord> cmp::Ord for StateCost<T> {
    // order by cost first (revert it to have the prio queue order by lowest first) and by state
    // secondary (state is not important for the priority queue)
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.cost.cmp(&other.cost) {
            cmp::Ordering::Equal => self.state.cmp(&other.state),
            o => o,
        }
        .reverse()
    }
}

impl<T: PartialOrd + Ord> cmp::PartialOrd for StateCost<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

const N_TYPES: usize = 4;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Burrow<const SIZE: usize, const N_AMPHIPODS: usize> {
    initial_state: [Option<usize>; SIZE],
    // edges and edge costs in thw burrow
    layout: [Vec<(usize, u32)>; SIZE],
    target_positions: [[usize; N_AMPHIPODS]; N_TYPES],
}

// create a small burrow (puzzle 1)
// positions are like this:
// #############
// #01.2.3.4.56#
// ###7#9#b#d###
//   #8#a#c#e#
//   #########
fn create_small_burrow(input_positions: &[usize; 8]) -> Burrow<15, 2> {
    Burrow {
        initial_state: [
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(input_positions[0]),
            Some(input_positions[1]),
            Some(input_positions[2]),
            Some(input_positions[3]),
            Some(input_positions[4]),
            Some(input_positions[5]),
            Some(input_positions[6]),
            Some(input_positions[7]),
        ],
        layout: [
            vec![(1, 1)],
            vec![(0, 1), (2, 2), (7, 2)],
            vec![(1, 2), (3, 2), (7, 2), (9, 2)],
            vec![(2, 2), (4, 2), (9, 2), (11, 2)],
            vec![(3, 2), (5, 2), (11, 2), (13, 2)],
            vec![(4, 2), (6, 1), (13, 2)],
            vec![(5, 1)],
            vec![(1, 2), (2, 2), (8, 1)],
            vec![(7, 1)],
            vec![(2, 2), (3, 2), (10, 1)],
            vec![(9, 1)],
            vec![(3, 2), (4, 2), (12, 1)],
            vec![(11, 1)],
            vec![(4, 2), (5, 2), (14, 1)],
            vec![(13, 1)],
        ],
        target_positions: [[7, 8], [9, 10], [11, 12], [13, 14]],
    }
}

// create a larger Burrow (puzzle 2)
// positions are like this:
// #############
// #01.2.3.4.56#
// ###7#b#f#j###
//   #8#c#g#k#
//   #9#d#h#l#
//   #a#e#i#m#
//   #########
fn create_large_burrow(input_positions: &[usize; 8]) -> Burrow<23, 4> {
    Burrow {
        initial_state: [
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(input_positions[0]),
            Some(3),
            Some(3),
            Some(input_positions[1]),
            Some(input_positions[2]),
            Some(2),
            Some(1),
            Some(input_positions[3]),
            Some(input_positions[4]),
            Some(1),
            Some(0),
            Some(input_positions[5]),
            Some(input_positions[6]),
            Some(0),
            Some(2),
            Some(input_positions[7]),
        ],
        layout: [
            // hallway
            vec![(1, 1)],
            vec![(0, 1), (2, 2), (7, 2)],
            vec![(1, 2), (3, 2), (7, 2), (11, 2)],
            vec![(2, 2), (4, 2), (11, 2), (15, 2)],
            vec![(3, 2), (5, 2), (15, 2), (19, 2)],
            vec![(4, 2), (6, 1), (19, 2)],
            vec![(5, 1)],
            // room A
            vec![(1, 2), (2, 2), (8, 1)],
            vec![(7, 1), (9, 1)],
            vec![(8, 1), (10, 1)],
            vec![(9, 1)],
            // room B
            vec![(2, 2), (3, 2), (12, 1)],
            vec![(11, 1), (13, 1)],
            vec![(12, 1), (14, 1)],
            vec![(13, 1)],
            // room C
            vec![(3, 2), (4, 2), (16, 1)],
            vec![(15, 1), (17, 1)],
            vec![(16, 1), (18, 1)],
            vec![(17, 1)],
            // room D
            vec![(4, 2), (5, 2), (20, 1)],
            vec![(19, 1), (21, 1)],
            vec![(20, 1), (22, 1)],
            vec![(21, 1)],
        ],
        target_positions: [
            [7, 8, 9, 10],
            [11, 12, 13, 14],
            [15, 16, 17, 18],
            [19, 20, 21, 22],
        ],
    }
}

const MOVE_COSTS: [u32; N_TYPES] = [1, 10, 100, 1000];

// parse input positions
// A -> 0 … D -> 3
// #############
// #...........#
// ###0#2#4#6###
//   #1#3#5#7#
//   #########
fn parse(input: &str) -> Result<[usize; 8], String> {
    let mut lines = input.lines();
    if lines.next() != Some("#############") || lines.next() != Some("#...........#") {
        return Err("unexpected hallways setup".to_owned());
    }

    let mut row1 = lines
        .next()
        .and_then(|line| line.strip_prefix("###"))
        .and_then(|line| line.strip_suffix("###"))
        .ok_or_else(|| "invalid format for first room row".to_owned())?
        .splitn(4, '#')
        .map(amphipod_shorthand_to_id);

    let mut row2 = lines
        .next()
        .and_then(|line| line.strip_prefix("  #"))
        .and_then(|line| line.strip_suffix('#'))
        .ok_or_else(|| "invalid format for first room row".to_owned())?
        .splitn(4, '#')
        .map(amphipod_shorthand_to_id);

    if lines.next() != Some("  #########") {
        return Err("unexpected bottom line in hallways setup".to_owned());
    }

    let result: [usize; 8] = [
        row1.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
        row2.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
        row1.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
        row2.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
        row1.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
        row2.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
        row1.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
        row2.next()
            .ok_or_else(|| "unable to find amphipod in row".to_owned())??,
    ];

    // sanity check: are there exactly two of each amphipod?
    let mut counts: [usize; 4] = [0; 4];
    for amphipod in result {
        counts[amphipod] += 1;
    }
    if counts != [2; 4] {
        return Err(format!("unexpected amphipod count in input: {:?}", counts));
    }

    Ok(result)
}

fn amphipod_shorthand_to_id(shorthand: &str) -> Result<usize, String> {
    match shorthand {
        "A" => Ok(0),
        "B" => Ok(1),
        "C" => Ok(2),
        "D" => Ok(3),
        _ => Err(format!("Unknown amphipod shorthand: '{}'", shorthand)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_DATA: &str = r"#############
#...........#
###B#C#B#D###
  #A#D#C#A#
  #########
";

    #[test]
    fn parse_works_for_example() {
        // when
        let result = parse(EXAMPLE_DATA);

        // then
        assert_eq!(result, Ok([1, 0, 2, 3, 1, 2, 3, 0]));
    }

    #[test]
    fn find_least_energy_works_for_example() {
        // given
        let initial_positions = parse(EXAMPLE_DATA).expect("expected successful parsing");
        let burrow = create_small_burrow(&initial_positions);

        // when
        let result = find_least_energy(&burrow);

        // then
        assert_eq!(result, Some(12521));
    }
}
