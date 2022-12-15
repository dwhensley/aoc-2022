use anyhow::{anyhow, Error, Result};

struct SectionRange {
    start: u64,
    end: u64,
}

impl SectionRange {
    fn new(start: u64, end: u64) -> Result<Self> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(anyhow!("Range start must be less than or equal to the end"))
        }
    }
}

impl TryFrom<&str> for SectionRange {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut s = value.split('-');
        let start = s
            .next()
            .ok_or_else(|| anyhow!("Could not parse start of range"))?
            .parse::<u64>()
            .map_err(|_e| anyhow!("Could not convert start of range to `u64`"))?;
        let end = s
            .next()
            .ok_or_else(|| anyhow!("Could not parse end of range"))?
            .parse::<u64>()
            .map_err(|_e| anyhow!("Could not convert end of range to `u64`"))?;
        Self::new(start, end)
            .map_err(|_e| anyhow!("Range start must be less than or equal to the end"))
    }
}

struct Assignment {
    elf1: SectionRange,
    elf2: SectionRange,
}

impl Assignment {
    fn containment(&self) -> bool {
        matches!((
            self.elf1.start,
            self.elf1.end,
            self.elf2.start,
            self.elf2.end,
        ), (s1, e1, s2, e2) if (s1 >= s2 && e1 <= e2) || (s1 <= s2 && e1 >= e2))
    }

    fn overlap(&self) -> bool {
        matches!((
            self.elf1.start,
            self.elf1.end,
            self.elf2.start,
            self.elf2.end,
        ), (s1, e1, s2, e2) if (s1 <= s2 && e1 >= s2) || (s2 <= s1 && e2 >= s1))
    }
}

fn read_assignments_p1(input: &str) -> Result<Vec<Assignment>> {
    let mut assignments = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        let mut s = line.split(',');
        let e1 = SectionRange::try_from(
            s.next()
                .ok_or_else(|| anyhow!("Could not parse first elf assignment"))?,
        )?;
        let e2 = SectionRange::try_from(
            s.next()
                .ok_or_else(|| anyhow!("Could not parse second elf assignment"))?,
        )?;
        assignments.push(Assignment { elf1: e1, elf2: e2 });
    }
    Ok(assignments)
}

fn main() {
    let fully_contained_count = read_assignments_p1("src/input.txt")
        .unwrap()
        .iter()
        .map(|a| a.containment())
        .filter(|b| *b)
        .count();
    println!("Part one: {fully_contained_count}");

    let overlap_count = read_assignments_p1("src/input.txt")
        .unwrap()
        .iter()
        .map(|a| a.overlap())
        .filter(|b| *b)
        .count();
    println!("Part two: {overlap_count}");
}

#[cfg(test)]
mod tests {
    use crate::read_assignments_p1;

    #[test]
    fn test_part_one() {
        assert_eq!(
            2,
            read_assignments_p1("src/test_input.txt")
                .unwrap()
                .iter()
                .map(|a| a.containment())
                .filter(|b| *b)
                .count()
        );
    }

    #[test]
    fn test_part_two() {
        assert_eq!(
            4,
            read_assignments_p1("src/test_input.txt")
                .unwrap()
                .iter()
                .map(|a| a.overlap())
                .filter(|b| *b)
                .count()
        )
    }
}
