use std::collections::HashSet;
use std::ops::Range;

use anyhow::{anyhow, Error, Result};

fn read_sensors(input: &str) -> Result<Vec<Sensor>> {
    let mut sensors = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        let mut s = line.split(": ");
        let sensor_info = s.next().ok_or_else(|| anyhow!("Expected sensor info"))?;
        let sensor_loc: Location = sensor_info
            .trim()
            .trim_start_matches("Sensor at ")
            .try_into()?;
        let beacon_info = s
            .next()
            .ok_or_else(|| anyhow!("Expected closest beacon info"))?;
        let beacon_loc: Location = beacon_info
            .trim()
            .trim_start_matches("closest beacon is at ")
            .try_into()?;
        sensors.push(Sensor::new(sensor_loc, Beacon::new(beacon_loc)));
    }
    Ok(sensors)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Location {
    x: isize,
    y: isize,
}

impl Location {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
    fn tuning_frequency(&self) -> isize {
        self.x * 4_000_000 + self.y
    }
}

impl TryFrom<&str> for Location {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut s = value.trim().split(", ");
        let x = s
            .next()
            .ok_or_else(|| anyhow!("Expected `x` location"))?
            .trim()
            .trim_start_matches("x=")
            .parse::<isize>()?;
        let y = s
            .next()
            .ok_or_else(|| anyhow!("Expected `y` location"))?
            .trim()
            .trim_start_matches("y=")
            .parse::<isize>()?;
        Ok(Self { x, y })
    }
}

impl Location {
    fn l1_dist(self, other: Location) -> usize {
        (self.x - other.x).unsigned_abs() + (self.y - other.y).unsigned_abs()
    }
}

#[derive(Debug, Copy, Clone)]
struct Beacon {
    loc: Location,
}

impl Beacon {
    fn new(loc: Location) -> Self {
        Self { loc }
    }
}

#[derive(Debug, Copy, Clone)]
struct Sensor {
    loc: Location,
    nearest_beacon: Beacon,
}

impl Sensor {
    fn new(loc: Location, nearest_beacon: Beacon) -> Self {
        Self {
            loc,
            nearest_beacon,
        }
    }
    fn beacon_l1(&self) -> usize {
        self.loc.l1_dist(self.nearest_beacon.loc)
    }
}

#[derive(Debug, Clone)]
struct Tunnels {
    sensors: Vec<Sensor>,
    beacons: HashSet<Location>,
}

impl Tunnels {
    fn new(sensors: Vec<Sensor>) -> Self {
        let mut beacons = HashSet::new();
        for sensor in sensors.iter() {
            let _ = beacons.insert(sensor.nearest_beacon.loc);
        }
        Self { sensors, beacons }
    }
    #[allow(dead_code)]
    fn row_info(&self) -> (isize, isize) {
        let mut min = isize::MAX;
        let mut max = isize::MIN;
        for sensor in &self.sensors {
            let ys = sensor.loc.y;
            let yb = sensor.nearest_beacon.loc.y;
            if ys < min {
                min = ys;
            } else if ys > max {
                max = ys;
            }
            if yb < min {
                min = yb;
            } else if yb > max {
                max = yb;
            }
        }
        (min, max)
    }

    fn col_info(&self) -> (isize, isize) {
        let mut min = isize::MAX;
        let mut max = isize::MIN;
        for sensor in &self.sensors {
            let xs = sensor.loc.x;
            let xb = sensor.nearest_beacon.loc.x;
            if xs < min {
                min = xs;
            } else if xs > max {
                max = xs;
            }
            if xb < min {
                min = xb;
            } else if xb > max {
                max = xb;
            }
        }
        (min, max)
    }

    fn beacon_possible(&self, loc: Location) -> bool {
        if self.beacons.contains(&loc) {
            return true;
        }
        for sensor in &self.sensors {
            if loc.l1_dist(sensor.loc) <= sensor.beacon_l1() {
                return false;
            }
        }
        true
    }

    fn impossible_ranges_in_row(
        &self,
        row: isize,
        min: isize,
        max: isize,
    ) -> Option<Vec<Range<isize>>> {
        let mut raw_ranges = Vec::new();
        for s in self.sensors.iter() {
            let dist_to_beacon = s.beacon_l1() as isize;
            let dist_to_row = (s.loc.y - row).abs();
            if dist_to_beacon >= dist_to_row {
                let diff = dist_to_beacon - dist_to_row;
                let start = std::cmp::max(s.loc.x - diff, min);
                let end = std::cmp::min(s.loc.x + diff, max);
                raw_ranges.push(start..end);
            }
        }
        if raw_ranges.is_empty() {
            None
        } else {
            raw_ranges.sort_unstable_by_key(|r| r.start);
            let mut merged_ranges = vec![raw_ranges[0].clone()];
            for rr in raw_ranges.into_iter().skip(1) {
                let prev = merged_ranges.last_mut().expect("Empty range when merging!");
                if rr.start + 1 > prev.end {
                    merged_ranges.push(rr);
                } else if rr.end > prev.end {
                    prev.end = rr.end;
                }
            }
            Some(merged_ranges)
        }
    }

    fn find_distress_beacon(
        &self,
        row_min: isize,
        row_max: isize,
        col_min: isize,
        col_max: isize,
    ) -> Option<Location> {
        'outer: for row in row_min..=row_max {
            let mut col = col_min;
            if let Some(ranges) = self.impossible_ranges_in_row(row, row_min, row_max) {
                for r in &ranges {
                    if r.contains(&col) {
                        let next = r.end + 1;
                        if next > col_max {
                            continue 'outer;
                        } else {
                            col = r.end + 1;
                        }
                    } else if !self.beacons.contains(&Location::new(col, row)) {
                        return Some(Location::new(col, row));
                    } else {
                        col += 1;
                        continue;
                    }
                }
            }
        }
        None
    }
}

fn main() -> Result<()> {
    let tunnels = Tunnels::new(read_sensors("src/input.txt").unwrap());
    let (col_min, col_max) = tunnels.col_info();
    let col_min = col_min - 2_000_000;
    let col_max = col_max + 2_000_000;
    let mut num_cannot_be_present = 0;
    for col in col_min..=col_max {
        if !tunnels.beacon_possible(Location::new(col, 2_000_000)) {
            num_cannot_be_present += 1;
        }
    }
    println!("Part one: {num_cannot_be_present}");

    // let tunnels = Tunnels::new(read_sensors("src/test_input.txt").unwrap());
    let col_min = 0isize;
    let col_max = 4_000_000isize;
    let row_min = 0isize;
    let row_max = 4_000_000isize;
    if let Some(distress_beacon_loc) =
        tunnels.find_distress_beacon(row_min, row_max, col_min, col_max)
    {
        let tuning_frequency = distress_beacon_loc.tuning_frequency();
        println!(
            "Part two: {tuning_frequency} at ({}, {})",
            distress_beacon_loc.x, distress_beacon_loc.y
        );
    } else {
        return Err(anyhow!("No distress beacon found!"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{read_sensors, Location, Tunnels};

    #[test]
    fn test_part_one() {
        let tunnels = Tunnels::new(read_sensors("src/test_input.txt").unwrap());
        let (col_min, col_max) = tunnels.col_info();
        let mut num_cannot_be_present = 0;
        for col in col_min..=col_max {
            if !tunnels.beacon_possible(Location::new(col, 10)) {
                num_cannot_be_present += 1;
            }
        }
        assert_eq!(26, num_cannot_be_present);
    }

    #[test]
    fn test_part_two() {
        let tunnels = Tunnels::new(read_sensors("src/test_input.txt").unwrap());
        let col_min = 0isize;
        let col_max = 20isize;
        let row_min = 0isize;
        let row_max = 20isize;
        let tuning_frequency = tunnels
            .find_distress_beacon(row_min, row_max, col_min, col_max)
            .unwrap()
            .tuning_frequency();
        assert_eq!(56_000_011, tuning_frequency);
    }
}
