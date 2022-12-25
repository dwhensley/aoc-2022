use std::collections::HashSet;

use anyhow::{anyhow, Result};

struct Assert<const COND: bool> {}

trait IsTrue {}

impl IsTrue for Assert<true> {}

#[derive(Debug, Copy, Clone)]
enum Move {
    Right(u8),
    Left(u8),
    Up(u8),
    Down(u8),
}

impl Move {
    fn try_from_line(line: &str) -> Result<Self> {
        let mut s = line.split(' ');
        let direction = s
            .next()
            .ok_or_else(|| anyhow!("Line is over but expected direction keyword"))?;
        let num_steps = s
            .next()
            .ok_or_else(|| anyhow!("Line is over but expected number of steps"))?
            .parse::<u8>()
            .map_err(|e| anyhow!("Could not parse {:?} as `u8`: {}", s, e))?;
        match (direction, num_steps) {
            ("R", n) => Ok(Self::Right(n)),
            ("L", n) => Ok(Self::Left(n)),
            ("U", n) => Ok(Self::Up(n)),
            ("D", n) => Ok(Self::Down(n)),
            (d, _) => Err(anyhow!("Unexpected direction keyword {}", d)),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default)]
struct Location {
    x: isize,
    y: isize,
}

#[derive(Debug, Clone)]
struct SimState<const N: usize> {
    locs: [Location; N],
    tl_loc_set: HashSet<Location>,
}

impl<const N: usize> SimState<N> {
    fn new() -> Self {
        SimState {
            locs: [Location::default(); N],
            tl_loc_set: HashSet::new(),
        }
    }
    fn advance_move(&mut self, mv: Move) {
        match mv {
            Move::Right(n) => {
                for _ in 0..n {
                    self.advance_right();
                }
            }
            Move::Left(n) => {
                for _ in 0..n {
                    self.advance_left();
                }
            }
            Move::Up(n) => {
                for _ in 0..n {
                    self.advance_up();
                }
            }
            Move::Down(n) => {
                for _ in 0..n {
                    self.advance_down();
                }
            }
        }
    }

    fn advance_right(&mut self) {
        self.locs[0].x += 1;
        self.advance();
    }

    fn advance_left(&mut self) {
        self.locs[0].x -= 1;
        self.advance();
    }

    fn advance_up(&mut self) {
        self.locs[0].y += 1;
        self.advance();
    }

    fn advance_down(&mut self) {
        self.locs[0].y -= 1;
        self.advance();
    }

    fn advance(&mut self) {
        for hd_idx in 0..N - 1 {
            self.update_tail_loc(hd_idx);
        }
        self.tl_loc_set.insert(self.locs[N - 1]);
    }

    fn update_tail_loc(&mut self, hd_idx: usize) {
        let (hd, tl) = (self.locs[hd_idx], self.locs[hd_idx + 1]);
        let x_diff = hd.x - tl.x;
        let y_diff = hd.y - tl.y;
        let tl_loc = &mut self.locs[hd_idx + 1];
        match (x_diff, y_diff) {
            (0, 0) | (1, 0) | (0, 1) | (-1, 0) | (0, -1) => {}
            (1, 1) | (1, -1) | (-1, 1) | (-1, -1) => {}
            (n, 0) if n >= 1 => tl_loc.x += 1,
            (n, 0) if n <= -1 => tl_loc.x -= 1,
            (0, n) if n >= 1 => tl_loc.y += 1,
            (0, n) if n <= -1 => tl_loc.y -= 1,
            (nx, ny) if nx.is_positive() && ny.is_positive() => {
                tl_loc.x += 1;
                tl_loc.y += 1;
            }
            (nx, ny) if nx.is_positive() && ny.is_negative() => {
                tl_loc.x += 1;
                tl_loc.y -= 1;
            }
            (nx, ny) if nx.is_negative() && ny.is_positive() => {
                tl_loc.x -= 1;
                tl_loc.y += 1;
            }
            (nx, ny) if nx.is_negative() && ny.is_negative() => {
                tl_loc.x -= 1;
                tl_loc.y -= 1;
            }
            (_, _) => {}
        }
    }
}

fn read_moves(input: &str) -> Result<Vec<Move>> {
    let mut moves = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        moves.push(Move::try_from_line(line)?);
    }
    Ok(moves)
}

fn main() -> Result<()> {
    let moves_list = read_moves("src/input.txt")?;
    let mut sim_p1 = SimState::<2>::new();
    for &m in moves_list.iter() {
        sim_p1.advance_move(m);
    }
    println!("Part one: {}", sim_p1.tl_loc_set.len());

    let mut sim_p2 = SimState::<10>::new();
    for &m in moves_list.iter() {
        sim_p2.advance_move(m);
    }
    println!("Part two: {}", sim_p2.tl_loc_set.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{read_moves, SimState};

    #[test]
    fn test_part_one() {
        let mut sim = SimState::<2>::new();
        for m in read_moves("src/test_input.txt").unwrap().into_iter() {
            sim.advance_move(m);
        }
        assert_eq!(13, sim.tl_loc_set.len());
    }

    #[test]
    fn test_part_two() {
        let mut sim = SimState::<10>::new();
        for m in read_moves("src/test_input2.txt").unwrap().into_iter() {
            sim.advance_move(m);
        }
        assert_eq!(36, sim.tl_loc_set.len());
    }
}
