use std::collections::HashSet;

use anyhow::Result;

fn read_data_stream(input: &str) -> Result<Vec<u8>> {
    Ok(std::fs::read_to_string(input)?.into_bytes())
}

fn all_unique(bytes: &[u8]) -> bool {
    if bytes.len() > 255 {
        false
    } else {
        let set: HashSet<u8> = HashSet::from_iter(bytes.iter().copied());
        set.len() == bytes.len()
    }
}

fn main() {
    let data_stream = read_data_stream("src/input.txt").unwrap();
    let mut char_count_p1 = 3;
    for w in data_stream.windows(4) {
        char_count_p1 += 1;
        if all_unique(w) {
            break;
        }
    }
    println!("Part one: {char_count_p1}");

    let mut char_count_p2 = 13;
    for w in data_stream.windows(14) {
        char_count_p2 += 1;
        if all_unique(w) {
            break;
        }
    }
    println!("Part two: {char_count_p2}");
}

#[cfg(test)]
mod tests {
    use crate::{all_unique, read_data_stream};

    #[test]
    fn test_part_one() {
        let data_stream = read_data_stream("src/test_input.txt").unwrap();
        let mut char_count = 3;
        for w in data_stream.windows(4) {
            char_count += 1;
            if all_unique(w) {
                break;
            }
        }
        assert_eq!(7, char_count);
    }

    #[test]
    fn test_part_two() {
        let data_stream = read_data_stream("src/test_input.txt").unwrap();
        let mut char_count = 13;
        for w in data_stream.windows(14) {
            char_count += 1;
            if all_unique(w) {
                break;
            }
        }
        assert_eq!(19, char_count);
    }
}
