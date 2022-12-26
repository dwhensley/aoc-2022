use std::{cmp::Ordering, collections::BinaryHeap};

use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone)]
struct Location {
    node_idx: usize,
    grid_idx: (usize, usize),
}

impl Location {
    fn new(node_idx: usize, grid_idx: (usize, usize)) -> Self {
        Self { node_idx, grid_idx }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct State {
    cost: usize,
    position: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Copy, Clone)]
struct Edge {
    node: usize,
    cost: usize,
}

fn shortest_path(adj_list: &Vec<Vec<Edge>>, start: usize, end: usize) -> Option<usize> {
    let mut dist: Vec<_> = (0..adj_list.len()).map(|_| usize::MAX).collect();
    let mut heap = BinaryHeap::new();

    dist[start] = 0;
    heap.push(State {
        cost: 0,
        position: start,
    });

    while let Some(State { cost, position }) = heap.pop() {
        if position == end {
            return Some(cost);
        }
        if cost > dist[position] {
            continue;
        }
        for edge in &adj_list[position] {
            let nxt = State {
                cost: cost + edge.cost,
                position: edge.node,
            };
            if nxt.cost < dist[nxt.position] {
                heap.push(nxt);
                dist[nxt.position] = nxt.cost;
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
struct HeightMap {
    grid: Vec<Vec<(Location, u8)>>,
}

impl HeightMap {
    fn new(grid: Vec<Vec<(Location, u8)>>) -> Self {
        Self { grid }
    }
    fn to_graph(&self) -> Vec<Vec<Edge>> {
        let mut graph = Vec::new();
        for ridx in 0..self.grid.len() {
            for cidx in 0..self.grid[ridx].len() {
                let mut nodes = Vec::new();
                let (_, curr) = self.grid[ridx][cidx];
                if ridx > 0 {
                    let (
                        Location {
                            node_idx,
                            grid_idx: _,
                        },
                        up,
                    ) = self.grid[ridx - 1][cidx];
                    if up as isize - curr as isize <= 1 {
                        nodes.push(Edge {
                            node: node_idx,
                            cost: 1,
                        });
                    }
                }
                if ridx <= self.grid.len() - 2 {
                    let (
                        Location {
                            node_idx,
                            grid_idx: _,
                        },
                        down,
                    ) = self.grid[ridx + 1][cidx];
                    if down as isize - curr as isize <= 1 {
                        nodes.push(Edge {
                            node: node_idx,
                            cost: 1,
                        });
                    }
                }
                if cidx > 0 {
                    let (
                        Location {
                            node_idx,
                            grid_idx: _,
                        },
                        left,
                    ) = self.grid[ridx][cidx - 1];
                    if left as isize - curr as isize <= 1 {
                        nodes.push(Edge {
                            node: node_idx,
                            cost: 1,
                        });
                    }
                }
                if cidx <= self.grid[ridx].len() - 2 {
                    let (
                        Location {
                            node_idx,
                            grid_idx: _,
                        },
                        right,
                    ) = self.grid[ridx][cidx + 1];
                    if right as isize - curr as isize <= 1 {
                        nodes.push(Edge {
                            node: node_idx,
                            cost: 1,
                        });
                    }
                }
                graph.push(nodes);
            }
        }
        graph
    }
    fn find_targets(&self, target: u8) -> Vec<Location> {
        self.grid
            .iter()
            .flatten()
            .filter_map(|&(l, h)| if h == target { Some(l) } else { None })
            .collect()
    }
}

fn read_heightmap(input: &str) -> Result<(Location, Location, HeightMap)> {
    let mut grid = Vec::new();
    let mut start_loc = Location::new(0, (0, 0));
    let mut end_loc = Location::new(0, (0, 0));
    let mut node_idx = 0;
    for (ridx, line) in std::fs::read_to_string(input)?.lines().enumerate() {
        let mut row = Vec::new();
        for (cidx, c) in line.chars().enumerate() {
            if !c.is_ascii_alphabetic() {
                return Err(anyhow!("Expected all ASCII alphabetic types, got {}", c));
            }
            if c == 'S' {
                start_loc.node_idx = node_idx;
                start_loc.grid_idx = (ridx, cidx);
                row.push((start_loc, b'a'));
            } else if c == 'E' {
                end_loc.node_idx = node_idx;
                end_loc.grid_idx = (ridx, cidx);
                row.push((end_loc, b'z'));
            } else {
                row.push((Location::new(node_idx, (ridx, cidx)), c as u8));
            }
            node_idx += 1;
        }
        grid.push(row);
    }
    Ok((start_loc, end_loc, HeightMap::new(grid)))
}

fn main() -> Result<()> {
    let (start, end, hmap) = read_heightmap("src/input.txt")?;
    let adj_list = hmap.to_graph();
    let p1 = shortest_path(&adj_list, start.node_idx, end.node_idx)
        .ok_or_else(|| anyhow!("No path found between {:?} and {:?}", start, end))?;
    println!("Part one: {p1}");

    let start_locs = hmap.find_targets(b'a');
    let p2 = start_locs
        .iter()
        .filter_map(|&start| shortest_path(&adj_list, start.node_idx, end.node_idx))
        .min()
        .unwrap();
    println!("Part two: {p2}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{read_heightmap, shortest_path};

    #[test]
    fn test_part_one() {
        let (start, end, hmap) = read_heightmap("src/test_input.txt").unwrap();
        let adj_list = hmap.to_graph();
        assert_eq!(
            31,
            shortest_path(&adj_list, start.node_idx, end.node_idx).unwrap()
        );
    }

    #[test]
    fn test_part_two() {
        let (_, end, hmap) = read_heightmap("src/test_input.txt").unwrap();
        let adj_list = hmap.to_graph();
        let start_locs = hmap.find_targets(b'a');
        assert_eq!(
            29,
            start_locs
                .iter()
                .filter_map(|&start| shortest_path(&adj_list, start.node_idx, end.node_idx))
                .min()
                .unwrap()
        );
    }
}
