use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone)]
struct CrateLocation {
    loc: usize,
    payload: char,
}

#[derive(Debug, Copy, Clone)]
struct StackMetaData {
    idx: usize,
    label: usize,
}

fn read_stacks_and_moves(input: &str) -> Result<(Vec<Stack>, Vec<Move>)> {
    let input = std::fs::read_to_string(input)?;
    let mut s = input.split("\n\n");
    let stack_data = s
        .next()
        .ok_or_else(|| anyhow!("Failed to parse initial stack data"))?;
    let mut parsed_crates = Vec::new();
    let mut parsed_meta: HashMap<usize, StackMetaData> = HashMap::new();
    for line in stack_data.lines() {
        let mut tokens = Vec::new();
        for (loc, c) in line.chars().enumerate() {
            if c.is_ascii() && !c.is_ascii_whitespace() {
                tokens.push((loc, c))
            }
        }
        if tokens.iter().all(|(_, c)| c.is_numeric()) {
            for (idx, &(loc, label)) in tokens.iter().enumerate() {
                let label = label
                    .to_digit(10)
                    .ok_or_else(|| anyhow!("Stack label is not a digit: {}", label))?
                    as usize;
                parsed_meta.insert(loc, StackMetaData { idx, label });
            }
            break;
        }
        let mut crates = Vec::new();
        let mut idx = 0;
        while idx < tokens.len() {
            let (_, c) = tokens[idx];
            if c == '[' && idx + 2 < tokens.len() {
                let (loc2, c2) = tokens[idx + 1];
                let (_, c3) = tokens[idx + 2];
                if c2.is_ascii_uppercase() && c3 == ']' {
                    crates.push(CrateLocation {
                        loc: loc2,
                        payload: c2,
                    });
                }
                idx += 3;
            }
        }
        if !crates.is_empty() {
            parsed_crates.push(crates);
        }
    }
    let parsed_stacks = parsed_crates_into_stacks(parsed_meta, parsed_crates)?;

    let move_data = s
        .next()
        .ok_or_else(|| anyhow!("Failed to parse initial moves data"))?;
    let mut parsed_moves = Vec::new();
    for line in move_data.lines() {
        let mut tokens = line.split(' ');
        let move_kw = tokens
            .next()
            .ok_or_else(|| anyhow!("Move line over but expected `move` keyword"))?;
        if move_kw != "move" {
            return Err(anyhow!("Expected `move` keyword, got {move_kw}"));
        }
        let num = tokens
            .next()
            .ok_or_else(|| anyhow!("Move line over but expected number of crates to move"))?
            .parse::<u8>()?;
        let from_kw = tokens
            .next()
            .ok_or_else(|| anyhow!("Move line over but expected `from` keyword"))?;
        if from_kw != "from" {
            return Err(anyhow!("Expected `from` keyword, got {from_kw}"));
        }
        let from_stack = tokens
            .next()
            .ok_or_else(|| anyhow!("Move line over but expected crate to move from"))?
            .parse::<u8>()?;
        let to_kw = tokens
            .next()
            .ok_or_else(|| anyhow!("Move line over but expected `to` keyword"))?;
        if to_kw != "to" {
            return Err(anyhow!("Expected `to` keyword, got {to_kw}"));
        }
        let to_stack = tokens
            .next()
            .ok_or_else(|| anyhow!("Move line over but expected crate to move to"))?
            .parse::<u8>()?;
        parsed_moves.push(Move {
            num,
            from: from_stack,
            to: to_stack,
        });
    }
    Ok((parsed_stacks, parsed_moves))
}

fn parsed_crates_into_stacks(
    parsed_meta: HashMap<usize, StackMetaData>,
    parsed_crates: Vec<Vec<CrateLocation>>,
) -> Result<Vec<Stack>> {
    let num_stacks = parsed_crates[parsed_crates.len() - 1].len();
    let mut stacks = vec![Stack::new(); num_stacks];
    for row in parsed_crates.iter().rev() {
        for crate_info in row.iter() {
            let stack_meta = parsed_meta
                .get(&crate_info.loc)
                .ok_or_else(|| anyhow!("Unexpected crate location: {}", &crate_info.loc))?;
            stacks[stack_meta.idx].push(crate_info.payload);
        }
    }
    Ok(stacks)
}

#[derive(Debug, Clone)]
struct Stack {
    contents: Vec<char>,
}

impl Stack {
    fn new() -> Self {
        Self {
            contents: Vec::new(),
        }
    }
    fn push(&mut self, c: char) {
        self.contents.push(c);
    }
    fn pop(&mut self) -> Option<char> {
        self.contents.pop()
    }
    fn top_element(&self) -> Option<char> {
        let len = self.contents.len();
        if len > 0 {
            Some(self.contents[len - 1])
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Move {
    num: u8,
    from: u8,
    to: u8,
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "move {} from {} to {}", self.num, self.from, self.to)
    }
}

impl Move {
    fn execute_9000(&self, stacks: &mut [Stack]) -> Result<()> {
        for _ in 0..self.num {
            let v = stacks[self.from as usize - 1]
                .pop()
                .ok_or_else(|| anyhow!("Can't pop an empty stack!"))?;
            stacks[self.to as usize - 1].push(v);
        }
        Ok(())
    }
    fn execute_9001(&self, stacks: &mut [Stack]) -> Result<()> {
        let mut buf = Vec::with_capacity(self.num as usize);
        for _ in 0..self.num {
            buf.push(
                stacks[self.from as usize - 1]
                    .pop()
                    .ok_or_else(|| anyhow!("Can't pop an empty stack!"))?,
            );
        }
        for v in buf.into_iter().rev() {
            stacks[self.to as usize - 1].push(v);
        }
        Ok(())
    }
}

fn main() {
    let (mut stacks_p1, moves) = read_stacks_and_moves("src/input.txt").unwrap();
    let mut stacks_p2 = stacks_p1.clone();
    for m in moves.iter() {
        m.execute_9000(&mut stacks_p1).unwrap();
    }
    let mut msg_p1 = String::new();
    for s in stacks_p1.iter() {
        if let Some(c) = s.top_element() {
            msg_p1.push(c);
        }
    }
    println!("Part one: {msg_p1}");

    for m in moves.iter() {
        m.execute_9001(&mut stacks_p2).unwrap();
    }
    let mut msg_p2 = String::new();
    for s in stacks_p2.iter() {
        if let Some(c) = s.top_element() {
            msg_p2.push(c);
        }
    }
    println!("Part two: {msg_p2}");
}

#[cfg(test)]
mod tests {
    use crate::read_stacks_and_moves;

    #[test]
    fn test_part_one() {
        let (mut stacks, moves) = read_stacks_and_moves("src/test_input.txt").unwrap();
        for m in moves.iter() {
            m.execute_9000(&mut stacks).unwrap();
        }
        let mut msg = String::new();
        for s in stacks.iter() {
            if let Some(c) = s.top_element() {
                msg.push(c);
            }
        }
        assert_eq!("CMZ", &msg);
    }

    #[test]
    fn test_part_two() {
        let (mut stacks, moves) = read_stacks_and_moves("src/test_input.txt").unwrap();
        for m in moves.iter() {
            m.execute_9001(&mut stacks).unwrap();
        }
        let mut msg = String::new();
        for s in stacks.iter() {
            if let Some(c) = s.top_element() {
                msg.push(c);
            }
        }
        assert_eq!("MCD", &msg);
    }
}
