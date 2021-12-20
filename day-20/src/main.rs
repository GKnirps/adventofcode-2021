use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let (image_enhancement, image) = parse(&content)?;

    let twice_enhanced_image = enhance_times(image, &image_enhancement, 2);
    println!(
        "After two enhancements, {} pixels are lit, out of bounds value is {}",
        lit_pixels(&twice_enhanced_image),
        twice_enhanced_image.out_of_bounds_value,
    );

    Ok(())
}

fn lit_pixels(image: &Image) -> usize {
    image.pixel.iter().filter(|p| **p != 0).count()
}

fn enhance_times(mut image: Image, lookup: &[u8], times: usize) -> Image {
    for _ in 0..times {
        image = enhance(&image, lookup);
    }
    image
}

// I should have made this a macro, so I could call it "enhance!".
fn enhance(image: &Image, lookup: &[u8]) -> Image {
    let new_width = image.width + 2;
    let new_height = (image.pixel.len() / image.width) + 2;
    let new_pixel: Vec<u8> = (0..(new_width * new_height))
        .map(|i| {
            let new_x = i % new_width;
            let new_y = i / new_width;
            let lookup_index = neighbour_code(image, new_x as isize - 1, new_y as isize - 1);
            lookup.get(lookup_index).copied().unwrap_or(0)
        })
        .collect();
    let out_of_bounds_value = if image.out_of_bounds_value == 0 {
        lookup.get(0).copied().unwrap_or(0)
    } else {
        lookup.get(511).copied().unwrap_or(0)
    };
    Image {
        width: new_width,
        pixel: new_pixel,
        out_of_bounds_value,
    }
}

const MOORE_NEIGHBOURS: [(isize, isize); 9] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

fn neighbour_code(image: &Image, px: isize, py: isize) -> usize {
    let mut code: usize = 0;
    for (dx, dy) in MOORE_NEIGHBOURS {
        code <<= 1;
        let x = px + dx;
        let y = py + dy;
        code |= if x >= 0
            && y >= 0
            && (x as usize) < image.width
            && (y as usize) < (image.pixel.len() / image.width)
        {
            image
                .pixel
                .get(x as usize + (y as usize * image.width))
                .copied()
                .unwrap_or(image.out_of_bounds_value) as usize
        } else {
            image.out_of_bounds_value as usize
        }
    }
    code
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Image {
    width: usize,
    pixel: Vec<u8>,
    out_of_bounds_value: u8,
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.pixel.chunks_exact(self.width) {
            for col in row {
                write!(f, "{}", if *col == 0 { '.' } else { '█' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn parse(input: &str) -> Result<(Vec<u8>, Image), String> {
    let (lookup, pixel) = input
        .split_once("\n\n")
        .ok_or_else(|| "unable to find enhancement algorithm and pixel".to_owned())?;
    let lookup: Vec<u8> = lookup
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '#' { 1 } else { 0 })
        .collect();

    if lookup.len() != 512 {
        return Err(format!(
            "Parse error: enhancement lookupt table has invalid size of {}",
            lookup.len()
        ));
    }

    let width = pixel
        .lines()
        .next()
        .ok_or_else(|| "Unable to find first line of image".to_owned())?
        .chars()
        .count();
    let pixel: Vec<u8> = pixel
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '#' { 1 } else { 0 })
        .collect();

    if pixel.len() % width != 0 {
        Err(format!(
            "number of pixel {} is not a multiple of the width {}",
            pixel.len(),
            width
        ))
    } else {
        Ok((
            lookup,
            Image {
                width,
                pixel,
                out_of_bounds_value: 0,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = r"..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..##
#..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###
.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#.
.#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#.....
.#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#..
...####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.....
..##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

#..#.
#....
##..#
..#..
..###
";

    #[test]
    fn neighbour_code_works_for_example() {
        // given
        let (_, image) = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let code = neighbour_code(&image, 2, 2);

        // then
        assert_eq!(code, 34);
    }

    #[test]
    fn neighbour_code_works_for_all_edge_cases() {
        // given
        let image = Image {
            width: 3,
            pixel: vec![1; 9],
        };
        let test_cases: &[(isize, isize, usize)] = &[
            (-1, -1, 0b000000001),
            (0, -1, 0b000000011),
            (1, -1, 0b000000111),
            (2, -1, 0b000000110),
            (3, -1, 0b000000100),
            (-1, 0, 0b000001001),
            (-1, 1, 0b001001001),
            (-1, 2, 0b001001000),
            (3, 0, 0b000100100),
            (3, 1, 0b100100100),
            (3, 2, 0b100100000),
            (-1, 3, 0b001000000),
            (0, 3, 0b011000000),
            (1, 3, 0b111000000),
            (2, 3, 0b110000000),
            (3, 3, 0b100000000),
        ];

        for (x, y, expected) in test_cases {
            // when
            let code = neighbour_code(&image, *x, *y);

            // then
            println!("testing {}×{}", x, y);
            assert_eq!(code, *expected);
        }
    }

    #[test]
    fn enhance_times_works_for_example() {
        // given
        let (lookup, image) = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let result = enhance_times(image, &lookup, 2);

        print!("{}", result);

        // then
        assert_eq!(lit_pixels(&result), 35);
    }
}
