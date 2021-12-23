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
    let room_layout = create_room_layout();

    print_pos(&initial_pos);

    if let Some(least_energy) = find_least_energy(initial_pos, &room_layout) {
        println!("The least energy to sort all amphipods is {}", least_energy);
    } else {
        println!("There is no way to get these amphipods to the right rooms.");
    }

    Ok(())
}

// so this is basically a shortest-path problems, but each vertice on the path is a constellation of
// amphipod positions
fn find_least_energy(initial_pos: Pos, room_layout: &RoomLayout) -> Option<u32> {
    let mut queue: BinaryHeap<StateCost> = BinaryHeap::with_capacity(128);
    queue.push(StateCost {
        state: initial_pos,
        cost: 0,
    });
    let mut seen: HashSet<Pos> = HashSet::with_capacity(256);

    while let Some(StateCost { state, cost }) = queue.pop() {
        if !seen.insert(state) {
            // already seen, do not need to look at again
            continue;
        }
        if is_final_pos(&state) {
            return Some(cost);
        }
        add_reachable_states(&mut queue, room_layout, state, cost);
    }
    None
}

fn add_reachable_states(
    queue: &mut BinaryHeap<StateCost>,
    room_layout: &RoomLayout,
    state: Pos,
    cost: u32,
) {
    for amph_index in 0..state.len() {
        // if the amphipod is already in the room where it wants to be (and no other amphipod types
        // need to get out), we don't need to look at that amphipod anymore
        if state[amph_index] == 8 + (amph_index / 2) * 2
            || (state[amph_index] == 7 + (amph_index / 2) * 2
                && state
                    .iter()
                    .enumerate()
                    .any(|(i, p)| i / 2 == amph_index / 2 && *p == 8 + (amph_index / 2) * 2))
        {
            continue;
        }
        for (p, p_cost) in explore_reachable_positions(room_layout, &state, amph_index) {
            let mut new_state = state;
            new_state[amph_index] = p;
            queue.push(StateCost {
                cost: cost + p_cost,
                state: new_state,
            });
        }
    }
}

fn explore_reachable_positions(
    room_layout: &RoomLayout,
    pos: &Pos,
    amph_index: usize,
) -> Vec<(usize, u32)> {
    let mut queue: VecDeque<(usize, u32)> = VecDeque::with_capacity(128);
    let mut seen: [bool; 15] = [false; 15];
    queue.push_back((pos[amph_index], 0));

    let mut reachable: Vec<(usize, u32)> = Vec::with_capacity(15);

    let movement_cost = MOVE_COST[amph_index];

    let outer_target_room = (amph_index / 2) * 2 + 7;
    let inner_target_room = (amph_index / 2) * 2 + 8;

    // we don't have uniform costs, but the room layout is a tree, so we still get an optimal
    // solution with a BFS
    while let Some((current, cost)) = queue.pop_front() {
        if seen[current] {
            continue;
        }
        seen[current] = true;

        // pos[amph_index] > 7 means the amphipod started in a room (and will move anywhere)
        // the other stuff means either the current room is the inner target room (we can/should move
        // there) or it is the outer target room and there is already the right amphipod in the
        // inner target room)
        // correct kind of amphipods
        if (pos[amph_index] >= 7 && current < 7)
            || current == inner_target_room
            || (current == outer_target_room
                && pos
                    .iter()
                    .enumerate()
                    .any(|(i, p)| *p == inner_target_room && i / 2 == amph_index / 2))
        {
            reachable.push((current, cost));
        }

        for (neighbour, cost_factor) in &room_layout[current] {
            if seen[*neighbour] {
                continue;
            }
            if pos.iter().any(|p| *p == *neighbour) {
                // space occupied, can't move here
                continue;
            }
            let neighbour_cost = cost + cost_factor * movement_cost;
            queue.push_back((*neighbour, neighbour_cost))
        }
    }
    reachable
}

fn is_final_pos(pos: &Pos) -> bool {
    ((pos[0] == 7 && pos[1] == 8) || (pos[0] == 8 && pos[1] == 7))
        && ((pos[2] == 9 && pos[3] == 10) || (pos[2] == 10 && pos[3] == 9))
        && ((pos[4] == 11 && pos[5] == 12) || (pos[4] == 12 && pos[5] == 11))
        && ((pos[6] == 13 && pos[7] == 14) || (pos[6] == 14 && pos[7] == 13))
}

// create a map of edges (and their cost factor) in the room constellation
// positions are like this
// #############
// #01.2.3.4.56#
// ###7#9#b#d###
//   #8#a#c#e#
//   #########
type RoomLayout = [Vec<(usize, u32)>; 15];
fn create_room_layout() -> RoomLayout {
    [
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
    ]
}

// struct with custom cmp operator for the priority queue
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
struct StateCost {
    cost: u32,
    state: Pos,
}

impl cmp::Ord for StateCost {
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

impl cmp::PartialOrd for StateCost {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// positions of all eight amphipods in the order [A, A, B, B, C, C, D, D]
// given the map, the positions number are like this (in hexadecimal)
// (positions directly in front of the rooms are ignored, as no one will stop there)
// #############
// #01.2.3.4.56#
// ###7#9#b#d###
//   #8#a#c#e#
//   #########
type Pos = [usize; 8];

fn print_pos(pos: &Pos) {
    let mut space: [char; 15] = ['.'; 15];
    space[pos[0]] = 'A';
    space[pos[1]] = 'A';
    space[pos[2]] = 'B';
    space[pos[3]] = 'B';
    space[pos[4]] = 'C';
    space[pos[5]] = 'C';
    space[pos[6]] = 'D';
    space[pos[7]] = 'D';
    println!(
        "#############\n#{}{}.{}.{}.{}.{}{}#\n###{}#{}#{}#{}###\n  #{}#{}#{}#{}#\n  #########",
        space[0],
        space[1],
        space[2],
        space[3],
        space[4],
        space[5],
        space[6],
        space[7],
        space[9],
        space[11],
        space[13],
        space[8],
        space[10],
        space[12],
        space[14]
    );
}

// energy cost for all eight amphipods in the same order their position is
const MOVE_COST: [u32; 8] = [1, 1, 10, 10, 100, 100, 1000, 1000];

fn parse(input: &str) -> Result<Pos, String> {
    let mut lines = input.lines();
    if lines.next() != Some("#############") || lines.next() != Some("#...........#") {
        return Err("unexpected hallways setup".to_owned());
    }

    let (pos, found_amber, found_bronze, found_copper, found_desert) = extract_amphipod_position(
        lines
            .next()
            .and_then(|line| line.strip_prefix("###"))
            .and_then(|line| line.strip_suffix("###"))
            .ok_or_else(|| "invalid format for first room row".to_owned())?,
        [0; 8],
        7,
        0,
        0,
        0,
        0,
    )?;

    let (pos, _, _, _, _) = extract_amphipod_position(
        lines
            .next()
            .and_then(|line| line.strip_prefix("  #"))
            .and_then(|line| line.strip_suffix("#"))
            .ok_or_else(|| "invalid format for first room row".to_owned())?,
        pos,
        8,
        found_amber,
        found_bronze,
        found_copper,
        found_desert,
    )?;

    if lines.next() != Some("  #########") {
        return Err("unexpected bottom line in hallways setup".to_owned());
    }

    // sanity check: are all amphipods in a room?
    if pos.iter().any(|p| *p == 0) {
        return Err(format!("Not all amphipods have a room: {:?}", pos));
    }

    Ok(pos)
}

fn extract_amphipod_position(
    line: &str,
    mut pos: Pos,
    pos_offset: usize,
    mut found_amber: usize,
    mut found_bronze: usize,
    mut found_copper: usize,
    mut found_desert: usize,
) -> Result<(Pos, usize, usize, usize, usize), String> {
    for (i, c) in line.splitn(4, '#').enumerate() {
        match c {
            "A" => {
                if found_amber > 1 {
                    return Err("found a third amber amphipod".to_owned());
                }
                pos[found_amber] = pos_offset + 2 * i;
                found_amber += 1;
            }
            "B" => {
                if found_bronze > 1 {
                    return Err("found a third bronze amphipod".to_owned());
                }
                pos[2 + found_bronze] = pos_offset + 2 * i;
                found_bronze += 1;
            }
            "C" => {
                if found_copper > 1 {
                    return Err("found a third copper amphipod".to_owned());
                }
                pos[4 + found_copper] = pos_offset + 2 * i;
                found_copper += 1;
            }
            "D" => {
                if found_desert > 1 {
                    return Err("found a third desert amphipod".to_owned());
                }
                pos[6 + found_desert] = pos_offset + 2 * i;
                found_desert += 1;
            }
            _ => {
                return Err(format!("Unexpected amphipod: '{}'", c));
            }
        }
    }
    Ok((pos, found_amber, found_bronze, found_copper, found_desert))
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
        assert_eq!(result, Ok([8, 14, 7, 11, 9, 12, 13, 10]));
    }

    #[test]
    fn find_least_energy_works_for_example() {
        // given
        let initial_positions = parse(EXAMPLE_DATA).expect("expected successful parsing");
        let room_layout = create_room_layout();

        // when
        let result = find_least_energy(initial_positions, &room_layout);

        // then
        assert_eq!(result, Some(12521));
    }
}
