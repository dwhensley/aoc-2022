use std::collections::HashSet;

use anyhow::{anyhow, Result};
use itertools::Itertools;

fn ascii_to_priority(c: char) -> u64 {
    if c.is_ascii_lowercase() {
        (c as u8 - 96) as u64
    } else {
        (c as u8 - 38) as u64
    }
}

struct RuckSack {
    c1: String,
    c2: String,
}

impl RuckSack {
    fn find_intersecting_item(&self) -> Result<char> {
        let c1_set: HashSet<char> = HashSet::from_iter(self.c1.chars());
        let c2_set: HashSet<char> = HashSet::from_iter(self.c2.chars());
        c1_set
            .intersection(&c2_set)
            .next()
            .copied()
            .ok_or_else(|| anyhow!("failed to find common item"))
    }
}

fn read_rucksack_list_p1(input: &str) -> Result<Vec<RuckSack>> {
    let mut sacks = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        let rucksack_size = line.len();
        let compartment_size = rucksack_size / 2;
        sacks.push(RuckSack {
            c1: line[0..compartment_size].to_string(),
            c2: line[compartment_size..rucksack_size].to_string(),
        });
    }
    Ok(sacks)
}

fn read_rucksack_list_p2(input: &str) -> Result<Vec<char>> {
    let mut group_badges: Vec<char> = Vec::new();
    for triple in std::fs::read_to_string(input)?
        .lines()
        .chunks(3)
        .into_iter()
    {
        let sets = triple
            .into_iter()
            .map(|v| HashSet::<char>::from_iter(v.chars()))
            .collect::<Vec<HashSet<char>>>();
        let tmp = sets[0]
            .intersection(&sets[1])
            .copied()
            .collect::<HashSet<char>>();
        let badge = tmp
            .intersection(&sets[2])
            .next()
            .copied()
            .ok_or_else(|| anyhow!("failed to find common badge"))?;
        group_badges.push(badge);
    }
    Ok(group_badges)
}

fn main() {
    let rucksack_list_p1 = read_rucksack_list_p1("src/input.txt").unwrap();
    let priority_sum = rucksack_list_p1
        .iter()
        .map(|r| {
            let c = r.find_intersecting_item().unwrap();
            ascii_to_priority(c)
        })
        .sum::<u64>();
    println!("Part one: {priority_sum}");

    let badge_priority_sum = read_rucksack_list_p2("src/input.txt")
        .unwrap()
        .iter()
        .copied()
        .map(ascii_to_priority)
        .sum::<u64>();
    println!("Part two: {badge_priority_sum}");
}

#[cfg(test)]
mod tests {
    use crate::{ascii_to_priority, read_rucksack_list_p1, read_rucksack_list_p2};

    #[test]
    fn test_part_one() {
        assert_eq!(
            157,
            read_rucksack_list_p1("src/test_input.txt")
                .unwrap()
                .iter()
                .map(|r| {
                    let c = r.find_intersecting_item().unwrap();
                    ascii_to_priority(c)
                })
                .sum::<u64>()
        )
    }

    #[test]
    fn test_part_two() {
        assert_eq!(
            70,
            read_rucksack_list_p2("src/test_input.txt")
                .unwrap()
                .iter()
                .copied()
                .map(ascii_to_priority)
                .sum::<u64>()
        );
    }
}
