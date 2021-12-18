use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let numbers = parse(&content)?;

    let summed_numbers = sum(&numbers);
    println!("The sum is {}", summed_numbers);
    println!("The magnitude of the sum is {}", magnitude(&summed_numbers));

    Ok(())
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum SnailfishNumber {
    Scal(u32),
    // I pobably could use an Rc here to avoid a lot of memory allocations but that would make code
    // that changes a number more complicated and I probably won't need the speed boost
    Pair(Box<(SnailfishNumber, SnailfishNumber)>),
}

impl fmt::Display for SnailfishNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SnailfishNumber::Scal(value) => write!(f, "{}", value),
            SnailfishNumber::Pair(sub) => write!(f, "[{},{}]", sub.0, sub.1),
        }
    }
}

fn magnitude(number: &SnailfishNumber) -> u32 {
    match number {
        SnailfishNumber::Scal(v) => *v,
        SnailfishNumber::Pair(sub) => 3 * magnitude(&sub.0) + 2 * magnitude(&sub.1),
    }
}

fn sum(numbers: &[SnailfishNumber]) -> SnailfishNumber {
    numbers
        .iter()
        .cloned()
        .reduce(add)
        .unwrap_or(SnailfishNumber::Scal(0))
}

fn add(left: SnailfishNumber, right: SnailfishNumber) -> SnailfishNumber {
    let mut sum = SnailfishNumber::Pair(Box::new((left, right)));

    reduce(&mut sum);

    sum
}

fn reduce(number: &mut SnailfishNumber) {
    let mut changed = true;
    while changed {
        let (exploded, _, _) = explode(number, 0);
        if !exploded {
            changed = split(number);
        }
    }
}

fn explode(number: &mut SnailfishNumber, layer: usize) -> (bool, Option<u32>, Option<u32>) {
    match number {
        SnailfishNumber::Scal(_) => (false, None, None),
        SnailfishNumber::Pair(sub) => {
            if layer >= 4 {
                if let (SnailfishNumber::Scal(left), SnailfishNumber::Scal(right)) = sub.as_mut() {
                    let left = *left;
                    let right = *right;
                    *number = SnailfishNumber::Scal(0);
                    return (true, Some(left), Some(right));
                }
            }
            let (exploded, left, right) = explode(&mut sub.0, layer + 1);
            if exploded {
                let right = if let Some(value) = right {
                    if add_leftmost_scalar(&mut sub.1, value) {
                        None
                    } else {
                        right
                    }
                } else {
                    None
                };
                return (true, left, right);
            }
            let (exploded, left, right) = explode(&mut sub.1, layer + 1);
            if exploded {
                let left = if let Some(value) = left {
                    if add_rightmost_scalar(&mut sub.0, value) {
                        None
                    } else {
                        left
                    }
                } else {
                    None
                };
                return (true, left, right);
            }
            (false, None, None)
        }
    }
}

fn add_rightmost_scalar(number: &mut SnailfishNumber, value: u32) -> bool {
    match number {
        SnailfishNumber::Scal(v) => {
            *number = SnailfishNumber::Scal(*v + value);
            true
        }
        SnailfishNumber::Pair(sub) => {
            if !add_rightmost_scalar(&mut sub.1, value) {
                add_rightmost_scalar(&mut sub.0, value)
            } else {
                true
            }
        }
    }
}

fn add_leftmost_scalar(number: &mut SnailfishNumber, value: u32) -> bool {
    match number {
        SnailfishNumber::Scal(v) => {
            *number = SnailfishNumber::Scal(*v + value);
            true
        }
        SnailfishNumber::Pair(sub) => {
            if !add_leftmost_scalar(&mut sub.0, value) {
                add_leftmost_scalar(&mut sub.1, value)
            } else {
                true
            }
        }
    }
}

fn split(number: &mut SnailfishNumber) -> bool {
    match number {
        SnailfishNumber::Scal(v) => {
            if *v >= 10 {
                *number = SnailfishNumber::Pair(Box::new((
                    SnailfishNumber::Scal(*v / 2),
                    SnailfishNumber::Scal(*v / 2 + *v % 2),
                )));
                true
            } else {
                false
            }
        }
        SnailfishNumber::Pair(sub) => split(&mut sub.0) || split(&mut sub.1),
    }
}

fn parse(input: &str) -> Result<Vec<SnailfishNumber>, String> {
    input
        .lines()
        .map(|line| parse_number(&mut line.chars()))
        .collect()
}

fn parse_number<T>(input: &mut T) -> Result<SnailfishNumber, String>
where
    T: Iterator<Item = char>,
{
    if let Some(c) = input.next() {
        // Assumption: All scalar values are only one digit long (everything else will be fail
        // parsing
        if let Some(scalar) = c.to_digit(10) {
            return Ok(SnailfishNumber::Scal(scalar));
        }
        if c != '[' {
            return Err(format!("Expected '[' but found '{}'", c));
        }
        let first_sub = parse_number(input)?;
        let separator = input.next();
        if separator != Some(',') {
            return Err(format!("Expected ',' but found '{:?}'", separator));
        }
        let second_sub = parse_number(input)?;
        let closing = input.next();
        if closing != Some(']') {
            return Err(format!("Expected ']' but found '{:?}'", closing));
        }
        Ok(SnailfishNumber::Pair(Box::new((first_sub, second_sub))))
    } else {
        Err("no input found".to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_number_works_for_example() {
        // given
        let input = "[[1,2],3]";

        // when
        let result = parse_number(&mut input.chars());

        // then
        assert_eq!(
            result,
            Ok(SnailfishNumber::Pair(Box::new((
                SnailfishNumber::Pair(Box::new((
                    SnailfishNumber::Scal(1),
                    SnailfishNumber::Scal(2)
                ))),
                SnailfishNumber::Scal(3)
            ))))
        );
    }

    #[test]
    fn explode_works_for_examples() {
        // given
        let input_output = &mut [
            (
                parse_number(&mut "[[[[[9,8],1],2],3],4]".chars()).unwrap(),
                "[[[[0,9],2],3],4]",
            ),
            (
                parse_number(&mut "[7,[6,[5,[4,[3,2]]]]]".chars()).unwrap(),
                "[7,[6,[5,[7,0]]]]",
            ),
            (
                parse_number(&mut "[[6,[5,[4,[3,2]]]],1]".chars()).unwrap(),
                "[[6,[5,[7,0]]],3]",
            ),
            (
                parse_number(&mut "[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]".chars()).unwrap(),
                "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
            ),
            (
                parse_number(&mut "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]".chars()).unwrap(),
                "[[3,[2,[8,0]]],[9,[5,[7,0]]]]",
            ),
        ];

        for (input, expected_output) in input_output {
            // when
            let (exploded, _, _) = explode(input, 0);

            // then
            assert!(exploded);
            assert_eq!(&input.to_string(), expected_output);
        }
    }

    #[test]
    fn split_works_for_examples() {
        // given
        let input_output = &mut [
            (SnailfishNumber::Scal(10), "[5,5]"),
            (SnailfishNumber::Scal(11), "[5,6]"),
            (
                SnailfishNumber::Pair(Box::new((
                    SnailfishNumber::Pair(Box::new((
                        SnailfishNumber::Scal(9),
                        SnailfishNumber::Scal(12),
                    ))),
                    SnailfishNumber::Scal(10),
                ))),
                "[[9,[6,6]],10]",
            ),
            (
                SnailfishNumber::Pair(Box::new((
                    SnailfishNumber::Pair(Box::new((
                        SnailfishNumber::Scal(9),
                        SnailfishNumber::Scal(1),
                    ))),
                    SnailfishNumber::Scal(11),
                ))),
                "[[9,1],[5,6]]",
            ),
        ];

        for (input, expected_output) in input_output {
            // when
            let did_split = split(input);

            // then
            assert!(did_split);
            assert_eq!(&input.to_string(), expected_output);
        }
    }

    #[test]
    fn add_works_for_examples() {
        // given
        let left = parse_number(&mut "[[[[4,3],4],4],[7,[[8,4],9]]]".chars())
            .expect("Expected successful parsing");
        let right = parse_number(&mut "[1,1]".chars()).expect("Expected successful parsing");

        // when
        let sum = add(left, right);

        // then
        assert_eq!(sum.to_string(), "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
    }

    // FIXME(GK)
    #[test]
    fn more_add_examples() {
        // given
        let left = parse_number(
            &mut "[[[[7,0],[7,7]],[[7,7],[7,8]]],[[[7,7],[8,8]],[[7,7],[8,7]]]]".chars(),
        )
        .expect("Expected successful parsing");
        let right = parse_number(&mut "[7,[5,[[3,8],[1,4]]]]".chars())
            .expect("Expected successful parsing");

        // when
        let sum = add(left, right);

        // then
        assert_eq!(
            sum.to_string(),
            "[[[[7,7],[7,8]],[[9,5],[8,7]]],[[[6,8],[0,8]],[[9,9],[9,0]]]]"
        );
    }

    #[test]
    fn sum_works_for_first_example() {
        // given
        let numbers = parse(
            r"[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
[7,[5,[[3,8],[1,4]]]]
[[2,[2,2]],[8,[8,1]]]
[2,9]
[1,[[[9,3],9],[[9,0],[0,7]]]]
[[[5,[7,4]],7],1]
[[[[4,2],2],6],[8,7]]
",
        )
        .expect("Expected successful parsing");

        // when
        let result = sum(&numbers);

        // then
        assert_eq!(
            result.to_string(),
            "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]"
        );
    }

    #[test]
    fn sum_works_for_second_example() {
        // given
        let numbers = parse(
            r"[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]
",
        )
        .expect("Expected successful parsing");

        // when
        let result = sum(&numbers);

        // then
        assert_eq!(
            result.to_string(),
            "[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]"
        );
    }
}
