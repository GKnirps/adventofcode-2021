use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let (random, cards) = parse(&content)?;

    if let Some(score) = play_bingo(cards, &random) {
        println!("The winning score is {}", score);
    } else {
        println!("There is no winner");
    }

    Ok(())
}

fn play_bingo(mut cards: Vec<Card>, random: &[u8]) -> Option<u32> {
    for number in random {
        for card in cards.iter_mut() {
            mark_number(card, *number);
            if card_has_won(card) {
                return Some(score(card, *number));
            }
        }
    }
    None
}

fn score(card: &Card, last_number: u8) -> u32 {
    card.fields
        .iter()
        .map(|(num, marked)| if !*marked { *num as u32 } else { 0u32 })
        .sum::<u32>()
        * (last_number as u32)
}

fn card_has_won(card: &Card) -> bool {
    let row_won = card
        .fields
        .chunks_exact(5)
        .any(|chunk| chunk.iter().all(|(_, marked)| *marked));
    if row_won {
        return true;
    }
    for col in 0..5 {
        let col_won = (0..5).all(|row| {
            card.fields
                .get(row * 5 + col)
                .map(|(_, marked)| *marked)
                .unwrap_or(false)
        });
        if col_won {
            return true;
        }
    }
    false
}

fn mark_number(card: &mut Card, number: u8) {
    for field in card.fields.iter_mut() {
        if field.0 == number {
            field.1 = true
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Card {
    // The size is constant, so we _could_ use an array here.
    // However, that makes it a bit harder to parse (we can't collect() into an array),
    // so here we are.
    fields: Vec<(u8, bool)>,
}

fn parse(input: &str) -> Result<(Vec<u8>, Vec<Card>), String> {
    let mut blocks = input.split("\n\n");
    let random = blocks
        .next()
        .ok_or_else(|| "expected a line of random numbers".to_owned())?
        .split(',')
        .map(|s| {
            s.parse::<u8>()
                .map_err(|e| format!("unable to parse 'random' number '{}': {}", s, e))
        })
        .collect::<Result<Vec<u8>, String>>()?;

    let cards = blocks
        .map(|block| {
            let fields = block
                .split_whitespace()
                .map(|s| {
                    let n: u8 = s
                        .parse()
                        .map_err(|e| format!("unable to parse number '{}' on a card: {}", s, e))?;
                    Ok((n, false))
                })
                .collect::<Result<Vec<(u8, bool)>, String>>()?;
            if fields.len() > 25 {
                return Err(format!(
                    "card has {} number on it, expected 25",
                    fields.len()
                ));
            }
            Ok(Card { fields })
        })
        .collect::<Result<Vec<Card>, String>>()?;

    Ok((random, cards))
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = r"7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

22 13 17 11  0
 8  2 23  4 24
21  9 14 16  7
 6 10  3 18  5
 1 12 20 15 19

 3 15  0  2 22
 9 18 13 17  5
19  8  7 25 23
20 11 10 24  4
14 21 16 12  6

14 21 17 24  4
10 16 15  9 19
18  8 23 26 20
22 11 13  6  5
 2  0 12  3  7";

    #[test]
    fn play_bingo_works_for_example() {
        // given
        let (random, cards) = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        let score = play_bingo(cards, &random);

        // then
        assert_eq!(score, Some(4512));
    }

    #[test]
    fn play_bingo_handles_missing_winner() {
        // given
        let (random, cards) = parse(EXAMPLE_INPUT).expect("Expected successful parsing");

        // when
        // in the exampl, the 12th number is the winning number
        let score = play_bingo(cards, &random[0..11]);

        // then
        assert_eq!(score, None);
    }

    #[test]
    fn card_has_won_recognizes_filled_column() {
        // given
        let card = Card {
            fields: vec![
                (0, false),
                (1, true),
                (2, false),
                (3, false),
                (4, false),
                (5, false),
                (6, true),
                (7, false),
                (8, false),
                (9, false),
                (10, false),
                (11, true),
                (12, false),
                (13, false),
                (14, false),
                (15, false),
                (16, true),
                (17, false),
                (18, false),
                (19, false),
                (20, false),
                (21, true),
                (22, false),
                (23, false),
                (24, false),
            ],
        };

        // when
        let won = card_has_won(&card);

        // then
        assert!(won);
    }
}
