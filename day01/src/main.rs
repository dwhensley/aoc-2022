use color_eyre::eyre::eyre;
use itertools::Itertools;

fn find_elf_with_max_calories(input: &str) -> color_eyre::Result<u64> {
    let max = std::fs::read_to_string(input)?
        .lines()
        .map(|v| v.parse::<u64>().ok())
        .batching(|it| {
            let mut sum = None;
            while let Some(Some(v)) = it.next() {
                sum = Some(sum.unwrap_or(0) + v);
            }
            sum
        })
        .max()
        .ok_or(eyre!("no summable calorie counts found"))?;

    Ok(max)
}

fn find_top_three_elf_calories(input: &str) -> color_eyre::Result<u64> {
    let mut top_three: [u64; 3] = [0; 3];
    for group in std::fs::read_to_string(input)?
        .replace("\r\n", "\n")
        .split("\n\n")
    {
        let mut sum = 0;
        for line in group.lines() {
            let value = line.parse::<u64>()?;
            sum += value;
        }
        if sum > top_three[0] {
            top_three[0] = sum;
        }
        top_three.sort();
    }
    Ok(top_three.iter().sum())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let max = find_elf_with_max_calories("src/input.txt")?;
    println!("Part one: {max}");

    let top_three = find_top_three_elf_calories("src/input.txt")?;
    println!("Part two: {top_three}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{find_elf_with_max_calories, find_top_three_elf_calories};

    #[test]
    fn test_part_one() {
        assert_eq!(
            24_000,
            find_elf_with_max_calories("src/test_input.txt").unwrap()
        );
    }

    #[test]
    fn test_part_two() {
        assert_eq!(
            45_000,
            find_top_three_elf_calories("src/test_input.txt").unwrap()
        );
    }
}
