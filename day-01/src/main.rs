use core::num::ParseIntError;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let input = content
        .lines()
        .map(|line| line.parse::<u32>())
        .collect::<Result<Vec<u32>, ParseIntError>>()
        .map_err(|e| e.to_string())?;

    let increased_depths = input.windows(2).filter(|win| win[0] < win[1]).count();

    println!(
        "{} measurements are larger than the previous one",
        increased_depths
    );

    let summed_depths: Vec<u32> = input.windows(3).map(|win| win.iter().sum()).collect();
    let increased_summed_depths = summed_depths
        .windows(2)
        .filter(|win| win[0] < win[1])
        .count();

    println!(
        "{} windowed measurements are larger than the previous one",
        increased_summed_depths
    );

    Ok(())
}
