use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let initial_pop = parse_population_by_timer(&content)?;

    let population_80 = run_days(initial_pop, 80);
    println!("After 80 days, there are {} lanternfish", population_80);

    Ok(())
}

fn run_days(mut population: [u32; 9], days: usize) -> u32 {
    for _ in 0..days {
        population = next_day(population);
    }
    population.iter().sum()
}

fn next_day(population: [u32; 9]) -> [u32; 9] {
    let mut next_pop: [u32; 9] = [0; 9];

    for i in 0..7 {
        next_pop[i] = population[(i + 1) % 7];
    }
    next_pop[6] += population[7];
    next_pop[7] = population[8];
    next_pop[8] = population[0];

    next_pop
}

fn parse_population_by_timer(input: &str) -> Result<[u32; 9], String> {
    let mut pop: [u32; 9] = [0; 9];
    for result in input.split(',').map(|s| {
        s.trim()
            .parse::<usize>()
            .map_err(|e| format!("Unable to parse input: {}", e))
    }) {
        // should have don this with a `fold()`…
        let timer = result?;
        if timer > 8 {
            return Err(format!("Expected no timer greater than 8, got {}", timer));
        }
        pop[timer] += 1;
    }
    Ok(pop)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_population_by_timer_works_for_example() {
        // given
        let input = "1,1,2,4,6,8,8\n";

        // when
        let pop = parse_population_by_timer(input);

        // then
        assert_eq!(pop, Ok([0, 2, 1, 0, 1, 0, 1, 0, 2]));
    }

    #[test]
    fn run_days_works_for_example() {
        // given
        let initial_population =
            parse_population_by_timer("3,4,3,1,2\n").expect("Expected sucessful parsing");

        // when
        let pop_count_18 = run_days(initial_population, 18);
        let pop_count_80 = run_days(initial_population, 80);

        // then
        assert_eq!(pop_count_18, 26);
        assert_eq!(pop_count_80, 5934);
    }
}
