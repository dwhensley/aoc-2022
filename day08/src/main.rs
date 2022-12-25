use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
struct Grid {
    shape: (usize, usize),
    trees: Vec<Vec<u8>>,
}

impl Grid {
    fn from_vecs(vecs: Vec<Vec<u8>>) -> Self {
        let num_rows = vecs.len();
        let num_cols = vecs[0].len();
        Self {
            shape: (num_rows, num_cols),
            trees: vecs,
        }
    }
    fn is_tree_visible(&self, row: usize, col: usize) -> bool {
        let height = self.trees[row][col];
        let taller_on_left = self.trees[row]
            .iter()
            .take(col)
            .map(|&t| t >= height)
            .filter(|b| *b)
            .count()
            > 0;
        if !taller_on_left {
            return true;
        }
        let taller_on_right = self.trees[row]
            .iter()
            .skip(col + 1)
            .map(|&t| t >= height)
            .filter(|b| *b)
            .count()
            > 0;
        if !taller_on_right {
            return true;
        }
        let taller_above = self
            .trees
            .iter()
            .take(row)
            .map(|r| r[col] >= height)
            .filter(|b| *b)
            .count()
            > 0;
        if !taller_above {
            return true;
        }
        let taller_below = self
            .trees
            .iter()
            .skip(row + 1)
            .map(|r| r[col] >= height)
            .filter(|b| *b)
            .count()
            > 0;
        if !taller_below {
            return true;
        }
        false
    }
    fn scenic_score(&self, row: usize, col: usize) -> usize {
        let height = self.trees[row][col];
        let mut trees_on_left = 0;
        let mut trees_on_right = 0;
        let mut trees_above = 0;
        let mut trees_below = 0;
        for &t in self.trees[row].iter().take(col).rev() {
            trees_on_left += 1;
            if t >= height {
                break;
            }
        }
        for &t in self.trees[row].iter().skip(col + 1) {
            trees_on_right += 1;
            if t >= height {
                break;
            }
        }
        for t in self.trees.iter().take(row).rev().map(|r| r[col]) {
            trees_above += 1;
            if t >= height {
                break;
            }
        }
        for t in self.trees.iter().skip(row + 1).map(|r| r[col]) {
            trees_below += 1;
            if t >= height {
                break;
            }
        }
        trees_on_left * trees_on_right * trees_above * trees_below
    }
}

fn read_grid(input: &str) -> Result<Grid> {
    let mut rows = Vec::new();
    for row in std::fs::read_to_string(input)?.lines() {
        let mut r = Vec::new();
        for c in row.chars() {
            if !c.is_ascii_digit() {
                return Err(anyhow!("Encountered non digit {}", c));
            } else {
                r.push(
                    c.to_digit(10)
                        .ok_or_else(|| anyhow!("Could not parse {} as digit", c))?
                        as u8,
                );
            }
        }
        rows.push(r);
    }
    Ok(Grid::from_vecs(rows))
}

fn main() {
    let grid = read_grid("src/input.txt").unwrap();
    let (nr, nc) = grid.shape;
    let mut num_visible_trees = 2 * nc + 2 * nr - 4;
    for r in 1..(nr - 1) {
        for c in 1..(nc - 1) {
            if grid.is_tree_visible(r, c) {
                num_visible_trees += 1;
            }
        }
    }
    println!("Part one: {num_visible_trees}");

    let mut top_scenic_score = 0;
    for r in 1..(nr - 1) {
        for c in 1..(nc - 1) {
            let scenic_score = grid.scenic_score(r, c);
            if scenic_score > top_scenic_score {
                top_scenic_score = scenic_score;
            }
        }
    }
    println!("Part two: {top_scenic_score}");
}

#[cfg(test)]
mod tests {
    use crate::read_grid;

    #[test]
    fn test_part_one() {
        let grid = read_grid("src/test_input.txt").unwrap();
        let (nr, nc) = grid.shape;
        let mut num_visible_trees = 2 * nc + 2 * nr - 4;
        for r in 1..(nr - 1) {
            for c in 1..(nc - 1) {
                if grid.is_tree_visible(r, c) {
                    num_visible_trees += 1;
                }
            }
        }
        assert_eq!(21, num_visible_trees);
    }

    #[test]
    fn test_part_two() {
        let grid = read_grid("src/test_input.txt").unwrap();
        let (nr, nc) = grid.shape;
        let mut top_scenic_score = 0;
        for r in 1..(nr - 1) {
            for c in 1..(nc - 1) {
                let scenic_score = grid.scenic_score(r, c);
                if scenic_score > top_scenic_score {
                    top_scenic_score = scenic_score;
                }
            }
        }
        assert_eq!(8, top_scenic_score);
    }
}
