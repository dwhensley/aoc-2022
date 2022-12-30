use std::fmt::Display;

use anyhow::{anyhow, Error, Result};

fn read_paths(input: &str) -> Result<Vec<RockPathKind>> {
    let mut paths = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        paths.extend_from_slice(&try_paths_from_line(line)?);
    }
    Ok(paths)
}

fn try_paths_from_line(line: &str) -> Result<Vec<RockPathKind>> {
    let mut paths = Vec::new();
    let coords = line.split(" -> ").collect::<Vec<&str>>();
    for (&start, &end) in coords.iter().zip(coords.iter().skip(1)) {
        paths.push(RockPathKind::new(
            Location::try_from(start)?,
            Location::try_from(end)?,
        )?);
    }
    Ok(paths)
}

#[derive(Debug, Copy, Clone)]
struct Location {
    x: isize,
    y: isize,
}

impl TryFrom<&str> for Location {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut s = value.trim().split(',');
        let x = s
            .next()
            .ok_or_else(|| anyhow!("Could not parse `x` coordinate from string"))?
            .parse::<isize>()?;
        let y = s
            .next()
            .ok_or_else(|| anyhow!("Could not parse `y` coordinate from string"))?
            .parse::<isize>()?;
        if let Some(s) = s.next() {
            Err(anyhow!(
                "Unexpected content after `x` and `y` parsed: {}",
                s
            ))
        } else {
            Ok(Self { x, y })
        }
    }
}

impl Location {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Copy, Clone)]
enum RockPathKind {
    Horizontal { start: Location, end: Location },
    Vertical { start: Location, end: Location },
}

impl RockPathKind {
    fn new(start: Location, end: Location) -> Result<Self> {
        match (start.x == end.x, start.y == end.y) {
            (true, true) => Err(anyhow!(
                "Rock path `start` and `end` are the same: ({}, {})",
                start.x,
                start.y
            )),
            (true, false) => {
                if end.y > start.y {
                    Ok(Self::Vertical { start, end })
                } else {
                    Ok(Self::Vertical {
                        start: end,
                        end: start,
                    })
                }
            }
            (false, true) => {
                if end.x > start.x {
                    Ok(Self::Horizontal { start, end })
                } else {
                    Ok(Self::Horizontal {
                        start: end,
                        end: start,
                    })
                }
            }
            (false, false) => Err(anyhow!(
                "Rock path ({}, {}) to ({}, {}) is not horizontal or vertical",
                start.x,
                start.y,
                end.x,
                end.y
            )),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Tile {
    Air,
    Rock,
    SandSource,
    SandAtRest,
}

#[derive(Debug, Copy, Clone)]
enum SandStatus {
    InMotion(Location),
    AtRest(Location),
}

#[derive(Debug, Copy, Clone)]
enum SimStatus {
    Continue,
    Overflow,
    Blocked,
}

#[derive(Debug, Clone)]
struct Cave {
    grid: Vec<Vec<Tile>>,
    source_loc: Location,
    x_lim: (isize, isize),
    y_lim: (isize, isize),
}

impl Cave {
    fn from_rock_path_list(
        mut rock_paths: Vec<RockPathKind>,
        source_loc: Location,
        base_layer: bool,
    ) -> Result<Self> {
        let (mut mn_x, mut mx_x) = (isize::MAX, isize::MIN);
        let (mn_y, mut mx_y) = (0, isize::MIN);

        for rp in rock_paths.iter() {
            match rp {
                RockPathKind::Horizontal { start, end } | RockPathKind::Vertical { start, end } => {
                    if start.x < mn_x {
                        mn_x = start.x;
                    } else if start.x > mx_x {
                        mx_x = start.x;
                    }
                    if end.x < mn_x {
                        mn_x = end.x;
                    } else if end.x > mx_x {
                        mx_x = end.x;
                    }
                    if start.y > mx_y {
                        mx_y = start.y;
                    }
                    if end.y > mx_y {
                        mx_y = end.y;
                    }
                }
            }
        }

        if base_layer {
            let extension = mn_x;
            rock_paths.push(RockPathKind::new(
                Location::new(mn_x - extension, mx_y + 2),
                Location::new(mx_x + extension, mx_y + 2),
            )?);
            mn_x -= extension;
            mx_x += extension;
            mx_y += 2;
        }
        let num_x = mx_x - mn_x + 1;
        let num_y = mx_y - mn_y + 1;
        let grid = vec![vec![Tile::Air; num_x as usize]; num_y as usize];
        let mut cave = Self {
            grid,
            source_loc,
            x_lim: (mn_x, mx_x),
            y_lim: (mn_y, mx_y),
        };
        for rp in rock_paths.iter() {
            match rp {
                RockPathKind::Horizontal { start, end } => {
                    let row = start.y;
                    for col in start.x..=end.x {
                        *cave.get_mut(Location::new(col, row))? = Tile::Rock;
                    }
                }
                RockPathKind::Vertical { start, end } => {
                    let col = start.x;
                    for row in start.y..=end.y {
                        *cave.get_mut(Location::new(col, row))? = Tile::Rock;
                    }
                }
            }
        }
        *cave.get_mut(source_loc)? = Tile::SandSource;
        Ok(cave)
    }
    fn check_coords(&self, loc: Location) -> Result<()> {
        let (x, y) = (loc.x, loc.y);
        let (x_min, x_max) = self.x_lim;
        let (y_min, y_max) = self.y_lim;
        match (y < y_min, y > y_max) {
            (true, _) => {
                return Err(anyhow!(
                    "`y` coordinate ({}) less than minimum ({})",
                    y,
                    y_min,
                ))
            }
            (_, true) => {
                return Err(anyhow!(
                    "`y` coordinate ({}) greater than maximum ({})",
                    y,
                    y_max,
                ))
            }
            _ => {}
        }
        match (x < x_min, x > x_max) {
            (true, _) => {
                return Err(anyhow!(
                    "`x` coordinate ({}) less than minimum ({})",
                    x,
                    x_min,
                ))
            }
            (_, true) => {
                return Err(anyhow!(
                    "`x` coordinate ({}) greater than maximum ({})",
                    x,
                    x_max,
                ))
            }
            _ => {}
        }
        Ok(())
    }
    fn get(&self, loc: Location) -> Result<Tile> {
        self.check_coords(loc)?;
        let (x_min, _) = self.x_lim;
        let (y_min, _) = self.y_lim;
        Ok(self.grid[(loc.y - y_min) as usize][(loc.x - x_min) as usize])
    }
    fn get_mut(&mut self, loc: Location) -> Result<&mut Tile> {
        self.check_coords(loc)?;
        let (x_min, _) = self.x_lim;
        let (y_min, _) = self.y_lim;
        Ok(self
            .grid
            .get_mut((loc.y - y_min) as usize)
            .expect("Failure with checked access")
            .get_mut((loc.x - x_min) as usize)
            .expect("Failure with checked access"))
    }
    fn sim_sand_drop(&mut self) -> Result<SimStatus> {
        let mut sand_status = SandStatus::InMotion(Location::new(500, 0));
        while let SandStatus::InMotion(loc) = sand_status {
            if let Tile::SandAtRest = self.get(self.source_loc)? {
                return Ok(SimStatus::Blocked);
            }
            let down_target = Location::new(loc.x, loc.y + 1);
            if let Ok(t) = self.get(down_target) {
                if let Tile::Air = t {
                    sand_status = SandStatus::InMotion(down_target);
                    continue;
                }
            } else {
                return Ok(SimStatus::Overflow);
            }
            let down_left_target = Location::new(loc.x - 1, loc.y + 1);
            if let Ok(t) = self.get(down_left_target) {
                if let Tile::Air = t {
                    sand_status = SandStatus::InMotion(down_left_target);
                    continue;
                }
            } else {
                return Ok(SimStatus::Overflow);
            }
            let down_right_target = Location::new(loc.x + 1, loc.y + 1);
            if let Ok(t) = self.get(down_right_target) {
                if let Tile::Air = t {
                    sand_status = SandStatus::InMotion(down_right_target);
                    continue;
                }
            } else {
                return Ok(SimStatus::Overflow);
            }
            sand_status = SandStatus::AtRest(loc);
            *self.get_mut(loc)? = Tile::SandAtRest;
        }
        Ok(SimStatus::Continue)
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for row in self.grid.iter() {
            writeln!(f)?;
            for tile in row.iter() {
                match tile {
                    Tile::Air => write!(f, ".")?,
                    Tile::Rock => write!(f, "#")?,
                    Tile::SandSource => write!(f, "+")?,
                    Tile::SandAtRest => write!(f, "o")?,
                };
            }
        }
        writeln!(f)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let rock_paths = read_paths("src/input.txt")?;
    let mut cave_p1 = Cave::from_rock_path_list(rock_paths.clone(), Location::new(500, 0), false)?;
    println!("Starting conditions:{}", &cave_p1);
    let mut num_sand_units_p1 = 0;
    while let SimStatus::Continue = cave_p1.sim_sand_drop()? {
        num_sand_units_p1 += 1;
    }
    println!("Ending state:{}", &cave_p1);
    println!("Part one: {num_sand_units_p1}");

    let mut cave_p2 = Cave::from_rock_path_list(rock_paths, Location::new(500, 0), true)?;
    let mut num_sand_units_p2 = 0;
    while let SimStatus::Continue = cave_p2.sim_sand_drop()? {
        num_sand_units_p2 += 1;
    }
    println!("Part two: {num_sand_units_p2}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{read_paths, Cave, Location, SimStatus};

    #[test]
    fn test_part_one() {
        let rock_paths = read_paths("src/test_input.txt").unwrap();
        let mut cave = Cave::from_rock_path_list(rock_paths, Location::new(500, 0), false).unwrap();
        let mut num_sand_units = 0;
        while let SimStatus::Continue = cave.sim_sand_drop().unwrap() {
            num_sand_units += 1;
        }
        assert_eq!(24, num_sand_units);
    }

    #[test]
    fn test_part_two() {
        let rock_paths = read_paths("src/test_input.txt").unwrap();
        let mut cave = Cave::from_rock_path_list(rock_paths, Location::new(500, 0), true).unwrap();
        let mut num_sand_units = 0;
        while let SimStatus::Continue = cave.sim_sand_drop().unwrap() {
            num_sand_units += 1;
        }
        assert_eq!(93, num_sand_units);
    }
}
