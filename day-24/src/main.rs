use std::cmp;
use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let program = parse(&content)?;

    let parameters = find_parameters_per_phase(&program)?;

    if let Some(largest_model_number) = brute_force_model_number(&parameters) {
        println!("Largest valid model number: {}", largest_model_number);
    } else {
        println!("There are no valid model numbers");
    }

    Ok(())
}

// I'm goint to regret brute forcing it but I'm tired and I want a solution
fn brute_force_model_number(parameters: &[(i64, i64, i64)]) -> Option<i64> {
    let mut number_by_z: HashMap<i64, i64> = HashMap::with_capacity(1);
    number_by_z.insert(0, 0);
    for (a, b, c) in parameters {
        let mut new_z_map: HashMap<i64, i64> = HashMap::with_capacity(number_by_z.len() * 9);
        for (z, model_number) in number_by_z {
            for input in 1..=9 {
                let new_z = simulate_block(z, *a, *b, *c, input);
                let new_number = model_number * 10 + input;
                let entry = new_z_map.entry(new_z).or_insert(new_number);
                *entry = cmp::max(*entry, new_number)
            }
        }
        number_by_z = new_z_map;
    }
    number_by_z
        .iter()
        .filter(|(z, _)| **z == 0)
        .map(|(_, number)| number)
        .max()
        .copied()
}

// simulate the repeating block (as seen in my input, see find_parameters_per_phase below)
// return the value of 'z' (all others are irrelevant as they are overwritten before they are read
// in each block)
fn simulate_block(z: i64, a: i64, b: i64, c: i64, input: i64) -> i64 {
    if z % 26 + b == input {
        z / a
    } else {
        (z / a) * 26 + input + c
    }
}

// this assumes a certain program structure that my input has.
// it will fail if the input has another structure
fn find_parameters_per_phase(program: &[Instruction]) -> Result<Vec<(i64, i64, i64)>, String> {
    program
        .split(|ins| ins == &Instruction::Inp(0))
        .filter(|block| !block.is_empty())
        .enumerate()
        .map(|(block_i, block)| {
            if block.len() != 17 {
                return Err(format!(
                    "unexpected block length in block {}: {}",
                    block_i,
                    block.len()
                ));
            }
            if block[0..3]
                != [
                    Instruction::Mul(1, Val::Lit(0)),
                    Instruction::Add(1, Val::Var(3)),
                    Instruction::Mod(1, Val::Lit(26)),
                ]
            {
                return Err(format!(
                    "unexpected instructions in line 0..3 in block {}",
                    block_i
                ));
            }
            let a = match block[3] {
                Instruction::Div(3, Val::Lit(a)) => a,
                _ => {
                    return Err(format!(
                        "unexpected instruction in line 3 in block {}",
                        block_i
                    ))
                }
            };
            let b = match block[4] {
                Instruction::Add(1, Val::Lit(b)) => b,
                _ => {
                    return Err(format!(
                        "unexpected instruction in line 4 in block {}",
                        block_i
                    ))
                }
            };
            if block[5..14]
                != [
                    Instruction::Eql(1, Val::Var(0)),
                    Instruction::Eql(1, Val::Lit(0)),
                    Instruction::Mul(2, Val::Lit(0)),
                    Instruction::Add(2, Val::Lit(25)),
                    Instruction::Mul(2, Val::Var(1)),
                    Instruction::Add(2, Val::Lit(1)),
                    Instruction::Mul(3, Val::Var(2)),
                    Instruction::Mul(2, Val::Lit(0)),
                    Instruction::Add(2, Val::Var(0)),
                ]
            {
                return Err(format!(
                    "unexpected instructions in line 5..14 in block {}",
                    block_i
                ));
            }
            let c = match block[14] {
                Instruction::Add(2, Val::Lit(c)) => c,
                _ => {
                    return Err(format!(
                        "unexpected instruction in line 14 in block {}",
                        block_i
                    ))
                }
            };
            if block[15..]
                != [
                    Instruction::Mul(2, Val::Var(1)),
                    Instruction::Add(3, Val::Var(2)),
                ]
            {
                return Err(format!(
                    "unexpected instructions in line 15.. in block {}",
                    block_i
                ));
            } else {
                Ok((a, b, c))
            }
        })
        .collect()
}

// run a program until it finishes or until it requires more input
// return (true, state) if the program finished and (false, state) if it did not finish and waits
// for more input
fn run_program(program: &[Instruction], mut state: Process, input: &[i64]) -> (bool, Process) {
    let mut input_stream = input.iter();
    loop {
        match program.get(state.ip).copied() {
            Some(Instruction::Inp(lhs)) => {
                if let Some(val) = input_stream.next() {
                    state.var[lhs] = *val;
                } else {
                    return (false, state);
                }
            }
            Some(Instruction::Add(lhs, rhs)) => {
                state.var[lhs] += get_value(&state, rhs);
            }
            Some(Instruction::Mul(lhs, rhs)) => {
                state.var[lhs] *= get_value(&state, rhs);
            }
            Some(Instruction::Div(lhs, rhs)) => {
                state.var[lhs] /= get_value(&state, rhs);
            }
            Some(Instruction::Mod(lhs, rhs)) => {
                state.var[lhs] %= get_value(&state, rhs);
            }
            Some(Instruction::Eql(lhs, rhs)) => {
                state.var[lhs] = if state.var[lhs] == get_value(&state, rhs) {
                    1
                } else {
                    0
                };
            }
            None => {
                return (true, state);
            }
        }
        state.ip += 1;
    }
}

fn get_value(state: &Process, val: Val) -> i64 {
    match val {
        Val::Lit(lit) => lit,
        Val::Var(i) => state.var[i],
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Process {
    var: [i64; 4],
    ip: usize,
}

impl Default for Process {
    fn default() -> Self {
        Process {
            var: [0, 0, 0, 0],
            ip: 0,
        }
    }
}

// all variables are matched to integers:
// w -> 0, x -> 1, y -> 2, z -> 3
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Val {
    Lit(i64),
    Var(usize),
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Instruction {
    Inp(usize),
    Add(usize, Val),
    Mul(usize, Val),
    Div(usize, Val),
    Mod(usize, Val),
    Eql(usize, Val),
}

fn parse(input: &str) -> Result<Vec<Instruction>, String> {
    input.lines().map(parse_instruction).collect()
}

fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let mut split = line.split_whitespace();
    let operator = split.next();
    let left = split
        .next()
        .ok_or_else(|| format!("expected at least one operand in line '{}'", line))?;
    let right = split.next();

    Ok(match operator {
        Some("inp") => Instruction::Inp(parse_variable(left)?),
        Some("add") => Instruction::Add(parse_variable(left)?, parse_val(right)?),
        Some("mul") => Instruction::Mul(parse_variable(left)?, parse_val(right)?),
        Some("div") => Instruction::Div(parse_variable(left)?, parse_val(right)?),
        Some("mod") => Instruction::Mod(parse_variable(left)?, parse_val(right)?),
        Some("eql") => Instruction::Eql(parse_variable(left)?, parse_val(right)?),
        _ => {
            return Err(format!("unknown operator '{:?}'", operator));
        }
    })
}

fn parse_val(val: Option<&str>) -> Result<Val, String> {
    let val = val.ok_or_else(|| "expected value, found nothing".to_owned())?;
    match val.parse::<i64>() {
        Ok(lit) => Ok(Val::Lit(lit)),
        _ => parse_variable(val)
            .map(Val::Var)
            .map_err(|_| format!("value '{}' is neither variable nor literal", val)),
    }
}

fn parse_variable(var: &str) -> Result<usize, String> {
    match var {
        "w" => Ok(0),
        "x" => Ok(1),
        "y" => Ok(2),
        "z" => Ok(3),
        _ => Err(format!("unknown variable: '{}'", var)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn run_program_works_for_example() {
        // given
        let program = parse(
            r"inp w
add z w
mod z 2
div w 2
add y w
mod y 2
div w 2
add x w
mod x 2
div w 2
mod w 2
",
        )
        .expect("expected successful parsing");

        let input = &[0b1010];

        // when
        let (finished, end_state) = run_program(&program, Process::default(), input);

        // then
        assert!(finished);
        assert_eq!(end_state.var, [1, 0, 1, 0]);
    }
}
