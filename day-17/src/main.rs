use std::cmp::min;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let target_area = parse(&content)?;

    if let Some(height) = find_highest_path(&target_area) {
        println!("Highest possible point: {}", height);
    } else {
        println!("I can't hit the target!");
    }

    Ok(())
}

fn find_highest_path(area: &Area) -> Option<i32> {
    // there is possibly a closed solution for that, but I'm gonna half-ass it and simulate it
    (area.bottom..=(area.bottom.abs()))
        // x and y are independent, so we can search for them independently and see where they match
        .filter(|vy| t_for_y_hits(*vy, area).iter().any(|t| t_hits_x(*t, area)))
        .max()
        .map(|vy| y_at_time(vy, vy))
}

fn t_hits_x(t: i32, area: &Area) -> bool {
    (0..=area.right).any(|vx| {
        let x = x_at_time(vx, t);
        x >= area.left && x <= area.right
    })
}

fn t_for_y_hits(vy: i32, area: &Area) -> Vec<i32> {
    (1i32..)
        .map(|t| (t, y_at_time(vy, t)))
        .take_while(|(_, y)| *y >= area.bottom)
        .filter(|(_, y)| *y <= area.top)
        .map(|(t, _)| t)
        .collect()
}

fn y_at_time(initial_speed: i32, t: i32) -> i32 {
    t * (initial_speed + 1) - ((t + 1) * t) / 2
}

fn x_at_time(initial_speed: i32, t: i32) -> i32 {
    let t = min(initial_speed.abs(), t);
    t * (initial_speed + 1) - initial_speed.signum() * ((t + 1) * t) / 2
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Area {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Projectile {
    pos_x: i32,
    pos_y: i32,
    vel_x: i32,
    vel_y: i32,
}

fn parse(input: &str) -> Result<Area, String> {
    let coords = input
        .strip_prefix("target area: ")
        .ok_or_else(|| "no target area in input".to_owned())?;
    let (xrange, yrange) = coords
        .split_once(", ")
        .ok_or_else(|| "unable to separate x range from y range".to_owned())?;
    let (left, right) = parse_range(xrange, "x=")?;
    let (bottom, top) = parse_range(yrange, "y=")?;

    Ok(Area {
        left,
        top,
        right,
        bottom,
    })
}

fn parse_range(input: &str, prefix: &str) -> Result<(i32, i32), String> {
    let (from, to) = input
        .strip_prefix(prefix)
        .ok_or_else(|| format!("range '{}' has not expected prefix '{}'", input, prefix))?
        .split_once("..")
        .ok_or_else(|| "unable to split range".to_owned())?;
    Ok((
        from.trim()
            .parse()
            .map_err(|e| format!("unable to parse '{}' in range '{}': {}", from, input, e))?,
        to.trim()
            .parse()
            .map_err(|e| format!("unable to parse '{}' in range '{}': {}", to, input, e))?,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_works_for_example() {
        // given
        let input = "target area: x=20..30, y=-10..-5\n";

        // when
        let result = parse(input);

        // then
        assert_eq!(
            result,
            Ok(Area {
                left: 20,
                right: 30,
                bottom: -10,
                top: -5
            })
        );
    }

    #[test]
    fn find_highest_path_works_for_example() {
        // given
        let area = Area {
            left: 20,
            right: 30,
            bottom: -10,
            top: -5,
        };

        // when
        let result = find_highest_path(&area);

        // then
        assert_eq!(result, Some(45));
    }

    #[test]
    fn test_x_at_time() {
        // given
        let initial_velocity = 3;

        // when/then
        assert_eq!(x_at_time(initial_velocity, 0), 0);
        assert_eq!(x_at_time(initial_velocity, 1), 3);
        assert_eq!(x_at_time(initial_velocity, 2), 5);
        assert_eq!(x_at_time(initial_velocity, 3), 6);
        assert_eq!(x_at_time(initial_velocity, 4), 6);
        assert_eq!(x_at_time(initial_velocity, 5), 6);
    }

    #[test]
    fn test_y_at_tim() {
        // given
        let initial_velocity = 3;

        // when/then
        assert_eq!(y_at_time(initial_velocity, 0), 0);
        assert_eq!(y_at_time(initial_velocity, 1), 3);
        assert_eq!(y_at_time(initial_velocity, 2), 5);
        assert_eq!(y_at_time(initial_velocity, 3), 6);
        assert_eq!(y_at_time(initial_velocity, 4), 6);
        assert_eq!(y_at_time(initial_velocity, 5), 5);
        assert_eq!(y_at_time(initial_velocity, 6), 3);
        assert_eq!(y_at_time(initial_velocity, 7), 0);
    }
}
