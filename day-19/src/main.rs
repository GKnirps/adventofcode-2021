use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let scanner_beacons = parse(&content)?;

    if let Some((matched_beacons, scanner_positions)) = match_all_scanners(&scanner_beacons) {
        let unique_beacons = get_unique_beacons(&matched_beacons);
        println!("There are {} unique beacons", unique_beacons.len());
        if let Some(longest_distance) = find_longest_scanner_distance(&scanner_positions) {
            println!(
                "The longest distance between two scanners is {}",
                longest_distance
            );
        } else {
            println!("There seem to be no scanners to measure a distance between them");
        }
    } else {
        println!("Unable to match all beacons. New probe required.");
    }

    Ok(())
}

fn find_longest_scanner_distance(scanner_positions: &[Pos]) -> Option<i64> {
    scanner_positions
        .iter()
        .flat_map(|left| {
            scanner_positions.iter().map(|right| {
                (left.x - right.x).abs() + (left.y - right.y).abs() + (left.z - right.z).abs()
            })
        })
        .max()
}

// requires all beacons to be in the same coordinate system
fn get_unique_beacons(scanner_beacons: &[Vec<Pos>]) -> HashSet<Pos> {
    scanner_beacons.iter().flatten().copied().collect()
}

fn match_all_scanners(scanner_beacons: &[Vec<Pos>]) -> Option<(Vec<Vec<Pos>>, Vec<Pos>)> {
    let mut matched_beacons: Vec<Vec<Pos>> = Vec::with_capacity(scanner_beacons.len());
    let mut scanner_positions: Vec<Pos> = Vec::with_capacity(scanner_beacons.len());
    matched_beacons.push(scanner_beacons.first()?.clone());
    scanner_positions.push(Pos::new(0, 0, 0));
    let mut reference_beacon_index: usize = 0;

    let mut unmatched_beacons: HashSet<&[Pos]> =
        scanner_beacons.iter().skip(1).map(|v| v.as_ref()).collect();

    while !unmatched_beacons.is_empty() {
        // if we can't get a new reference beacon, we can't match anything new, but we have not
        // matched all unmatched beacons yet, so we can't match all scanners
        let reference_beacon = matched_beacons.get(reference_beacon_index)?;
        let newly_matched: Vec<(&[Pos], Vec<Pos>, Pos)> = unmatched_beacons
            .iter()
            .filter_map(|ub| {
                match_scanners(reference_beacon, ub)
                    .map(|(normalized, scanner_pos)| (*ub, normalized, scanner_pos))
            })
            .collect();
        for (raw, normalized, scanner_pos) in newly_matched {
            unmatched_beacons.remove(raw);
            matched_beacons.push(normalized);
            scanner_positions.push(scanner_pos);
        }
        reference_beacon_index += 1;
    }
    Some((matched_beacons, scanner_positions))
}

// tries to match the beacons of two scanners, for a match, at least 12 beacons must match
// returns the second set of beacons transformed into the coordinate system of the first one on
// success or None if they do not match
fn match_scanners(beacons1: &[Pos], beacons2: &[Pos]) -> Option<(Vec<Pos>, Pos)> {
    // phew… I can already how this becomes a huge performance issue
    for permut_i in 0..N_PERMUTATIONS {
        for base_beacon1 in beacons1 {
            for base_beacon2 in beacons2.iter().map(|b| b.permute(permut_i)) {
                let distance = base_beacon2 - *base_beacon1;
                let matching_beacons_count = beacons2
                    .iter()
                    .map(|b| b.permute(permut_i))
                    .filter(|b2| beacons1.iter().any(|b1| *b1 == *b2 - distance))
                    .count();
                if matching_beacons_count >= 12 {
                    // match!
                    return Some((
                        beacons2
                            .iter()
                            .map(|b| b.permute(permut_i) - distance)
                            .collect(),
                        Pos::new(0, 0, 0) - distance,
                    ));
                }
            }
        }
    }
    None
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Pos {
    x: i64,
    y: i64,
    z: i64,
}

impl std::ops::Add for Pos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Pos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Pos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

const N_PERMUTATIONS: usize = 24;
impl Pos {
    fn new(x: i64, y: i64, z: i64) -> Self {
        Pos { x, y, z }
    }
    // permutes the vector by one of the given permutations (i < 24, will wrap around if higher)
    fn permute(self, i: usize) -> Pos {
        let Pos {
            x: px,
            y: py,
            z: pz,
        } = self;
        match i % N_PERMUTATIONS {
            0 => Pos::new(px, py, pz),
            1 => Pos::new(px, -pz, py),
            2 => Pos::new(px, -py, -pz),
            3 => Pos::new(px, pz, -py),

            4 => Pos::new(-px, py, -pz),
            5 => Pos::new(-px, -pz, -py),
            6 => Pos::new(-px, -py, pz),
            7 => Pos::new(-px, pz, py),

            8 => Pos::new(py, pz, px),
            9 => Pos::new(py, -px, pz),
            10 => Pos::new(py, -pz, -px),
            11 => Pos::new(py, px, -pz),

            12 => Pos::new(-py, px, pz),
            13 => Pos::new(-py, pz, -px),
            14 => Pos::new(-py, -px, -pz),
            15 => Pos::new(-py, -pz, px),

            16 => Pos::new(pz, px, py),
            17 => Pos::new(pz, -py, px),
            18 => Pos::new(pz, -px, -py),
            19 => Pos::new(pz, py, -px),

            20 => Pos::new(-pz, py, px),
            21 => Pos::new(-pz, px, -py),
            22 => Pos::new(-pz, -py, -px),
            23 => Pos::new(-pz, -px, py),
            _ => panic!("This is actually an exhaustive pattern since i % 24 is always in 0..=23"),
        }
    }
}

fn parse(input: &str) -> Result<Vec<Vec<Pos>>, String> {
    input
        .split("\n\n")
        .map(|block| {
            let mut lines = block.lines();
            let header_line = lines
                .next()
                .ok_or_else(|| "expected header line for scanner".to_owned())?;
            if !header_line.starts_with("--- scanner ") || !header_line.ends_with(" ---") {
                return Err(format!(
                    "unexpected format for scanner header line: '{}'",
                    header_line
                ));
            }
            lines.map(parse_pos).collect::<Result<Vec<Pos>, String>>()
        })
        .collect()
}

fn parse_pos(input: &str) -> Result<Pos, String> {
    let mut coords = input.splitn(3, ',').map(|s| {
        s.parse::<i64>()
            .map_err(|e| format!("unable to pars ordinate '{}': {}", s, e))
    });
    let x = coords
        .next()
        .ok_or_else(|| format!("expected a value for x in line '{}'", input))??;
    let y = coords
        .next()
        .ok_or_else(|| format!("expected a value for y in line '{}'", input))??;
    let z = coords
        .next()
        .ok_or_else(|| format!("expected a value for z in line '{}'", input))??;
    Ok(Pos { x, y, z })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pos_permute_returns_different_results_for_all_permutations() {
        // given
        let pos = Pos {
            x: 1,
            y: 10,
            z: 100,
        };

        // when
        let result: HashSet<Pos> = (0..24).map(|i| pos.permute(i)).collect();

        // then
        assert_eq!(result.len(), 24);
        for pos in result {
            assert_ne!(pos.x, pos.y);
            assert_ne!(pos.x, pos.z);
            assert_ne!(pos.y, pos.z);
        }
    }

    #[test]
    fn match_scanners_works_for_example() {
        // given
        let scanners = parse(EXAMPLE_INPUT).expect("Expected successful parsing");
        assert_eq!(scanners.len(), 5);

        // when
        let result = match_scanners(&scanners[0], &scanners[1]);

        // then
        let (_, scanner_pos) = result.expect("expected match");
        assert_eq!(scanner_pos, Pos::new(68, -1246, -43));
    }

    #[test]
    fn match_all_scanners_works_for_example() {
        // given
        let scanners = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let matched_scanners = match_all_scanners(&scanners);

        // then
        let (beacons, scanner_positions) = matched_scanners.expect("expected successful matching");
        let unique_beacons = get_unique_beacons(&beacons);
        assert_eq!(unique_beacons.len(), 79);
        assert_eq!(
            find_longest_scanner_distance(&scanner_positions),
            Some(3621)
        );
    }

    const EXAMPLE_INPUT: &str = r"--- scanner 0 ---
404,-588,-901
528,-643,409
-838,591,734
390,-675,-793
-537,-823,-458
-485,-357,347
-345,-311,381
-661,-816,-575
-876,649,763
-618,-824,-621
553,345,-567
474,580,667
-447,-329,318
-584,868,-557
544,-627,-890
564,392,-477
455,729,728
-892,524,684
-689,845,-530
423,-701,434
7,-33,-71
630,319,-379
443,580,662
-789,900,-551
459,-707,401

--- scanner 1 ---
686,422,578
605,423,415
515,917,-361
-336,658,858
95,138,22
-476,619,847
-340,-569,-846
567,-361,727
-460,603,-452
669,-402,600
729,430,532
-500,-761,534
-322,571,750
-466,-666,-811
-429,-592,574
-355,545,-477
703,-491,-529
-328,-685,520
413,935,-424
-391,539,-444
586,-435,557
-364,-763,-893
807,-499,-711
755,-354,-619
553,889,-390

--- scanner 2 ---
649,640,665
682,-795,504
-784,533,-524
-644,584,-595
-588,-843,648
-30,6,44
-674,560,763
500,723,-460
609,671,-379
-555,-800,653
-675,-892,-343
697,-426,-610
578,704,681
493,664,-388
-671,-858,530
-667,343,800
571,-461,-707
-138,-166,112
-889,563,-600
646,-828,498
640,759,510
-630,509,768
-681,-892,-333
673,-379,-804
-742,-814,-386
577,-820,562

--- scanner 3 ---
-589,542,597
605,-692,669
-500,565,-823
-660,373,557
-458,-679,-417
-488,449,543
-626,468,-788
338,-750,-386
528,-832,-391
562,-778,733
-938,-730,414
543,643,-506
-524,371,-870
407,773,750
-104,29,83
378,-903,-323
-778,-728,485
426,699,580
-438,-605,-362
-469,-447,-387
509,732,623
647,635,-688
-868,-804,481
614,-800,639
595,780,-596

--- scanner 4 ---
727,592,562
-293,-554,779
441,611,-461
-714,465,-776
-743,427,-804
-660,-479,-426
832,-632,460
927,-485,-438
408,393,-506
466,436,-512
110,16,151
-258,-428,682
-393,719,612
-211,-452,876
808,-476,-593
-575,615,604
-485,667,467
-680,325,-822
-627,-443,-432
872,-547,-609
833,512,582
807,604,487
839,-516,451
891,-625,532
-652,-548,-490
30,-46,-14
";
}
