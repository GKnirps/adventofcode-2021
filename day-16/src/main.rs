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

    Ok(())
}

fn sum_versions(packet: &Packet) -> u32 {
    packet.version as u32
        + match &packet.t {
            PacketType::Op { sub } => sub.iter().map(sum_versions).sum(),
            PacketType::Lit { data: _ } => 0,
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
            _ => parse_operator(input.get(6..).ok_or_else(|| {
                "Unexpected end of input while reading operator packer".to_owned()
            })?)
            .map(|(sub, read)| {
                (
                    Packet {
                        version,
                        t: PacketType::Op { sub },
                    },
                    read,
                )
            }),
        }?;
    Ok((packet, (6 + read_length)))
}

fn parse_literal(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let mut data: Vec<u8> = Vec::with_capacity(16);
    let mut more_packages = true;
    let mut offset: usize = 0;
    while more_packages {
        let bits = input
            .get(offset..(offset + 5))
            .ok_or_else(|| "Unexpected end of input while reading data".to_owned())?;
        offset += 5;
        more_packages = bits[0] != 0;
        data.push(bits[1] << 3 | bits[2] << 2 | bits[3] << 1 | bits[4]);
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
    Lit { data: Vec<u8> },
    Op { sub: Vec<Packet> },
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
                t: PacketType::Lit {
                    data: vec![0b0111, 0b1110, 0b0101]
                }
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
                    sub: vec![
                        Packet {
                            version: 6,
                            t: PacketType::Lit { data: vec![0b1010] }
                        },
                        Packet {
                            version: 2,
                            t: PacketType::Lit {
                                data: vec![0b0001, 0b0100]
                            }
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
                    sub: vec![
                        Packet {
                            version: 2,
                            t: PacketType::Lit { data: vec![0b0001] }
                        },
                        Packet {
                            version: 4,
                            t: PacketType::Lit { data: vec![0b0010] }
                        },
                        Packet {
                            version: 1,
                            t: PacketType::Lit { data: vec![0b0011] }
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
}
