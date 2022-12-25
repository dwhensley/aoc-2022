use std::collections::VecDeque;

use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone)]
enum Operation {
    Mul(usize),
    Add(usize),
    Square,
    Double,
}

#[derive(Debug, Clone)]
struct Monkey {
    items: VecDeque<usize>,
    operation: Operation,
    divisor: usize,
    true_midx: usize,
    false_midx: usize,
    inspection_count: usize,
}

impl Monkey {
    fn new(
        items: VecDeque<usize>,
        operation: Operation,
        divisor: usize,
        true_midx: usize,
        false_midx: usize,
    ) -> Self {
        Self {
            items,
            operation,
            divisor,
            true_midx,
            false_midx,
            inspection_count: 0,
        }
    }
    fn pop(&mut self) -> Option<usize> {
        self.items.pop_front()
    }
    fn push(&mut self, item: usize) {
        self.items.push_back(item)
    }
    fn try_from_str(s: &str) -> Result<Self> {
        let mut items = VecDeque::new();
        let mut lines = s.split('\n');
        let monkey_label = lines
            .next()
            .ok_or_else(|| anyhow!("Raw monkey block ended but expected monkey label"))?;
        if !monkey_label.starts_with("Monkey ") {
            return Err(anyhow!("Expected monkey label but got {}", &monkey_label));
        }
        let start = lines
            .next()
            .ok_or_else(|| anyhow!("Raw monkey block ended but expected starting items line"))?;
        let start_items = start.trim().trim_start_matches("Starting items: ");
        for item in start_items.split(", ") {
            items.push_back(
                item.parse::<usize>()
                    .map_err(|e| anyhow!("Failed to parse starting item {} -- {}", item, e))?,
            );
        }
        let operation = lines
            .next()
            .ok_or_else(|| anyhow!("Raw monkey block ended but expected operation line"))?;
        let mut op_pair = operation
            .trim()
            .trim_start_matches("Operation: new = old ")
            .split(' ');
        let op_ty = op_pair
            .next()
            .ok_or_else(|| anyhow!("Operation line over but expected operator type"))?;
        let op_arg = op_pair
            .next()
            .ok_or_else(|| anyhow!("Operation line ended but expected operator argument"))?;
        let operation = match (op_ty, op_arg) {
            ("*", "old") => Operation::Square,
            ("+", "old") => Operation::Double,
            ("*", a) => Operation::Mul(a.parse::<usize>()?),
            ("+", a) => Operation::Add(a.parse::<usize>()?),
            (o, a) => return Err(anyhow!("Unexpected operator {} with target {}", o, a)),
        };
        let test_ln = lines
            .next()
            .ok_or_else(|| anyhow!("Raw monkey block ended but expected test case line"))?;
        let divisor = test_ln
            .trim()
            .trim_start_matches("Test: divisible by ")
            .parse::<usize>()?;
        let true_case = lines
            .next()
            .ok_or_else(|| anyhow!("Raw monkey block ended but expected true test case"))?;
        let true_monkey_idx = true_case
            .trim()
            .trim_start_matches("If true: throw to monkey ")
            .parse::<usize>()?;
        let false_case = lines
            .next()
            .ok_or_else(|| anyhow!("Raw monkey block ended but expected false test case"))?;
        let false_monkey_idx = false_case
            .trim()
            .trim_start_matches("If false: throw to monkey ")
            .parse::<usize>()?;
        Ok(Monkey::new(
            items,
            operation,
            divisor,
            true_monkey_idx,
            false_monkey_idx,
        ))
    }
}

#[derive(Debug, Clone)]
struct MonkeyShow {
    monkeys: Box<[Monkey]>,
    test_product: usize,
}

impl MonkeyShow {
    fn new(monkeys: Box<[Monkey]>) -> Self {
        let test_product = monkeys.iter().map(|v| v.divisor).product::<usize>();
        Self {
            monkeys,
            test_product,
        }
    }
    fn exe_round(&mut self, relief: bool) {
        for midx in 0..self.monkeys.len() {
            while let Some(mut item) = self.monkeys[midx].pop() {
                self.monkeys[midx].inspection_count += 1;
                match self.monkeys[midx].operation {
                    Operation::Mul(a) => item *= a,
                    Operation::Add(a) => item += a,
                    Operation::Square => item *= item,
                    Operation::Double => item += item,
                }
                if relief {
                    item /= 3;
                } else {
                    item %= self.test_product;
                }
                let to_idx = if item % self.monkeys[midx].divisor == 0 {
                    self.monkeys[midx].true_midx
                } else {
                    self.monkeys[midx].false_midx
                };
                self.monkeys[to_idx].push(item);
            }
        }
    }
    fn monkey_business(&self) -> usize {
        let mut fst = 0usize;
        let mut snd = 0usize;
        for v in self.monkeys.iter().map(|v| v.inspection_count) {
            if v > fst {
                snd = fst;
                fst = v;
            } else if v > snd {
                snd = v;
            }
        }
        fst * snd
    }
}

fn read_initial_state(input: &str) -> Result<MonkeyShow> {
    let mut monkeys = Vec::new();
    for raw_monkey in std::fs::read_to_string(input)?.split("\n\n") {
        monkeys.push(Monkey::try_from_str(raw_monkey)?);
    }
    Ok(MonkeyShow::new(monkeys.into_boxed_slice()))
}

fn main() -> Result<()> {
    let mut show_p1 = read_initial_state("src/input.txt")?;
    let mut show_p2 = show_p1.clone();
    for _ in 0..20 {
        show_p1.exe_round(true);
    }
    println!("Part one: {}", show_p1.monkey_business());

    for _ in 0..10_000 {
        show_p2.exe_round(false);
    }
    println!("Part two: {}", show_p2.monkey_business());
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::read_initial_state;

    #[test]
    fn test_part_one() {
        let mut show = read_initial_state("src/test_input.txt").unwrap();
        for _ in 0..20 {
            show.exe_round(true);
        }
        assert_eq!(10_605, show.monkey_business());
    }

    #[test]
    fn test_part_two() {
        let mut show = read_initial_state("src/test_input.txt").unwrap();
        for _ in 0..10_000 {
            show.exe_round(false);
        }
        assert_eq!(2_713_310_158, show.monkey_business());
    }
}
