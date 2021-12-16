use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let packet = parse(&content)?;

    let version_sum = sum_versions(&packet);
    println!("The version sum is {}", version_sum);

    let result = eval(&packet);
    println!("The evaluated result is {}", result);

    Ok(())
}

fn eval(packet: &Packet) -> u64 {
    match &packet.t {
        PacketType::Lit { data } => *data,
        PacketType::Op { oc, sub } => {
            let mut operands = sub.iter().map(eval);
            match oc {
                Opcode::Sum => operands.sum(),
                Opcode::Prod => operands.product(),
                // the specification did not say what to do if there is no operand for min or max, so I
                // assume it is undefined behaviour and I can do what I want
                Opcode::Min => operands.min().unwrap_or(0),
                Opcode::Max => operands.max().unwrap_or(0),
                // the specification said those three always have exactly two operands, so again I
                // assume nasal demons are allowed if they don't
                Opcode::Gt => {
                    if operands.next().unwrap_or(0) > operands.next().unwrap_or(0) {
                        1
                    } else {
                        0
                    }
                }
                Opcode::Lt => {
                    if operands.next().unwrap_or(0) < operands.next().unwrap_or(0) {
                        1
                    } else {
                        0
                    }
                }
                Opcode::Equal => {
                    if operands.next().unwrap_or(0) == operands.next().unwrap_or(0) {
                        1
                    } else {
                        0
                    }
                }
            }
        }
    }
}

fn sum_versions(packet: &Packet) -> u32 {
    packet.version as u32
        + match &packet.t {
            PacketType::Op { sub, .. } => sub.iter().map(sum_versions).sum(),
            PacketType::Lit { .. } => 0,
        }
}

fn parse(input: &str) -> Result<Packet, String> {
    let bits: Vec<u8> = input
        .chars()
        .filter_map(|c| c.to_digit(16).map(|d| d as u8))
        .flat_map(|d| [d >> 3, (d >> 2) & 1, (d >> 1) & 1, d & 1])
        .collect();
    parse_packet(&bits).map(|(p, _)| p)
}

fn parse_packet(input: &[u8]) -> Result<(Packet, usize), String> {
    let version_bits = input
        .get(0..3)
        .ok_or_else(|| "Unexpected end of input while reading packet version".to_owned())?;
    let version: u8 = version_bits[0] << 2 | version_bits[1] << 1 | version_bits[2];
    let type_bits = input
        .get(3..6)
        .ok_or_else(|| "Unexpected end of input while reading packet type".to_owned())?;
    let (packet, read_length) =
        match type_bits {
            &[1, 0, 0] => parse_literal(input.get(6..).ok_or_else(|| {
                "Unexpected end of input while reading literal packet".to_owned()
            })?)
            .map(|(data, read)| {
                (
                    Packet {
                        version,
                        t: PacketType::Lit { data },
                    },
                    read,
                )
            }),
            _ => {
                let opcode = match *type_bits {
                    [0, 0, 0] => Opcode::Sum,
                    [0, 0, 1] => Opcode::Prod,
                    [0, 1, 0] => Opcode::Min,
                    [0, 1, 1] => Opcode::Max,
                    [1, 0, 1] => Opcode::Gt,
                    [1, 1, 0] => Opcode::Lt,
                    [1, 1, 1] => Opcode::Equal,
                    _ => panic!("unhandled type bits"),
                };
                parse_operator(input.get(6..).ok_or_else(|| {
                    "Unexpected end of input while reading operator packer".to_owned()
                })?)
                .map(|(sub, read)| {
                    (
                        Packet {
                            version,
                            t: PacketType::Op { oc: opcode, sub },
                        },
                        read,
                    )
                })
            }
        }?;
    Ok((packet, (6 + read_length)))
}

fn parse_literal(input: &[u8]) -> Result<(u64, usize), String> {
    let mut data: u64 = 0;
    let mut more_packages = true;
    let mut offset: usize = 0;
    while more_packages {
        let bits = input
            .get(offset..(offset + 5))
            .ok_or_else(|| "Unexpected end of input while reading data".to_owned())?;
        offset += 5;
        more_packages = bits[0] != 0;
        data = data << 4 | (bits[1] << 3 | bits[2] << 2 | bits[3] << 1 | bits[4]) as u64;
    }
    Ok((data, offset))
}

fn parse_operator(input: &[u8]) -> Result<(Vec<Packet>, usize), String> {
    let length_type = *input
        .get(0)
        .ok_or_else(|| "Unexpected end of input while reading operator length type".to_owned())?;
    if length_type == 0 {
        // length is in bits
        let length = bits_to_usize(input.get(1..16).ok_or_else(|| {
            "Unexpected end of input while reading operator length in bits".to_owned()
        })?);
        let mut offset = 16;
        let mut sub: Vec<Packet> = Vec::with_capacity(8);
        while offset < length + 16 {
            let (packet, bits_read) = parse_packet(input.get(offset..).ok_or_else(|| {
                "Unexpected end of input while reading sublice for sub packet".to_owned()
            })?)?;
            offset += bits_read;
            sub.push(packet);
        }
        if offset != length + 16 {
            Err("Sub package read more bits than expected".to_owned())
        } else {
            Ok((sub, offset))
        }
    } else {
        // length is in number of sub packages
        let length = bits_to_usize(input.get(1..12).ok_or_else(|| {
            "Unexpected end of input while reading operator length in packets".to_owned()
        })?);
        let mut offset = 12;
        let mut sub: Vec<Packet> = Vec::with_capacity(length);
        for _ in 0..length {
            let (packet, bits_read) = parse_packet(input.get(offset..).ok_or_else(|| {
                "Unexpected end of input while reading sublice for sub packet".to_owned()
            })?)?;
            offset += bits_read;
            sub.push(packet);
        }
        Ok((sub, offset))
    }
}

fn bits_to_usize(input: &[u8]) -> usize {
    let mut result: usize = 0;
    for bit in input {
        result <<= 1;
        result |= *bit as usize;
    }
    result
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Packet {
    version: u8,
    t: PacketType,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum PacketType {
    // the specification said nothing about a maximal size of a number, so I originally used
    // Vec<u8>
    // u64 is easier to handle, so let's hope it is sufficient
    Lit { data: u64 },
    Op { oc: Opcode, sub: Vec<Packet> },
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Opcode {
    Sum,
    Prod,
    Min,
    Max,
    Gt,
    Lt,
    Equal,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_works_for_literal_example() {
        // given
        let input = "D2FE28";

        // when
        let result = parse(input);

        // then
        assert_eq!(
            result,
            Ok(Packet {
                version: 6,
                t: PacketType::Lit { data: 2021 }
            })
        );
    }

    #[test]
    fn parse_works_for_first_operator_example() {
        // given
        let input = "38006F45291200";

        // when
        let result = parse(input);

        // then
        assert_eq!(
            result,
            Ok(Packet {
                version: 1,
                t: PacketType::Op {
                    oc: Opcode::Lt,
                    sub: vec![
                        Packet {
                            version: 6,
                            t: PacketType::Lit { data: 10 }
                        },
                        Packet {
                            version: 2,
                            t: PacketType::Lit { data: 20 }
                        },
                    ]
                }
            })
        );
    }

    #[test]
    fn parse_works_for_second_operator_example() {
        // given
        let input = "EE00D40C823060";

        // when
        let result = parse(input);

        // then
        assert_eq!(
            result,
            Ok(Packet {
                version: 7,
                t: PacketType::Op {
                    oc: Opcode::Max,
                    sub: vec![
                        Packet {
                            version: 2,
                            t: PacketType::Lit { data: 1 }
                        },
                        Packet {
                            version: 4,
                            t: PacketType::Lit { data: 2 }
                        },
                        Packet {
                            version: 1,
                            t: PacketType::Lit { data: 3 }
                        },
                    ]
                }
            })
        );
    }

    #[test]
    fn sum_versions_works_for_last_example() {
        // given
        let packet = parse("A0016C880162017C3686B18A3D4780").expect("Expected successful parsing");

        // when
        let result = sum_versions(&packet);

        // then
        assert_eq!(result, 31);
    }

    #[test]
    fn eval_works_for_last_example() {
        // given
        let packet = parse("9C0141080250320F1802104A08").expect("Expected successful parsing");

        // when
        let result = eval(&packet);

        // then
        assert_eq!(result, 1);
    }
}
