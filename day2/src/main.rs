use anyhow::Result;

fn read_guide_p1(input: &str) -> Result<Vec<Game>> {
    let mut game = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        let mut moves = line.split(' ');
        let opponent = moves.next();
        let player = moves.next();
        match (opponent, player) {
            (Some(o), Some(p)) => {
                let opponent = RpsMove::try_from(o);
                let player = RpsMove::try_from(p);
                match (opponent, player) {
                    (Ok(o), Ok(p)) => {
                        game.push(Game {
                            player_move: p,
                            opponent_move: o,
                        });
                    }
                    (_, _) => continue,
                }
            }
            (_, _) => continue,
        }
    }
    Ok(game)
}

fn read_guide_p2(input: &str) -> Result<Vec<Game>> {
    let mut game = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        let mut moves = line.split(' ');
        let opponent = moves.next();
        let player_strat = moves.next();
        match (opponent, player_strat) {
            (Some(o), Some(p)) => {
                let opponent = RpsMove::try_from(o);
                let player_strat = RoundStrategy::try_from(p);
                match (opponent, player_strat) {
                    (Ok(o), Ok(p)) => {
                        game.push(Game {
                            player_move: p.play(&o),
                            opponent_move: o,
                        });
                    }
                    (_, _) => continue,
                }
            }
            (_, _) => continue,
        }
    }
    Ok(game)
}

#[derive(Debug, Copy, Clone)]
enum RoundStrategy {
    Lose,
    Draw,
    Win,
}

impl TryFrom<&str> for RoundStrategy {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "X" => Ok(Self::Lose),
            "Y" => Ok(Self::Draw),
            "Z" => Ok(Self::Win),
            _ => Err("Could not parse round strategy"),
        }
    }
}

impl RoundStrategy {
    fn play(&self, opponent: &RpsMove) -> RpsMove {
        match self {
            Self::Lose => opponent.to_lose(),
            Self::Draw => opponent.to_draw(),
            Self::Win => opponent.to_win(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum RpsMove {
    R = 1,
    P = 2,
    S = 3,
}

impl TryFrom<&str> for RpsMove {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "A" | "X" => Ok(Self::R),
            "B" | "Y" => Ok(Self::P),
            "C" | "Z" => Ok(Self::S),
            _ => Err("Could not parse RPS move"),
        }
    }
}

impl RpsMove {
    fn to_win(self) -> Self {
        match self {
            Self::R => Self::P,
            Self::P => Self::S,
            Self::S => Self::R,
        }
    }
    fn to_draw(self) -> Self {
        self
    }
    fn to_lose(self) -> Self {
        match self {
            Self::R => Self::S,
            Self::P => Self::R,
            Self::S => Self::P,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Game {
    player_move: RpsMove,
    opponent_move: RpsMove,
}

impl Game {
    fn score(&self) -> u8 {
        match (self.opponent_move, self.player_move) {
            (RpsMove::R, p @ RpsMove::P) => 6 + p as u8,
            (RpsMove::R, p @ RpsMove::R) => 3 + p as u8,
            (RpsMove::P, p @ RpsMove::S) => 6 + p as u8,
            (RpsMove::P, p @ RpsMove::P) => 3 + p as u8,
            (RpsMove::S, p @ RpsMove::R) => 6 + p as u8,
            (RpsMove::S, p @ RpsMove::S) => 3 + p as u8,
            (_, p) => p as u8,
        }
    }
}

fn main() -> Result<()> {
    let guide_p1 = read_guide_p1("src/input.txt").unwrap();
    let total_score_p1: u64 = guide_p1.iter().map(|g| g.score() as u64).sum();
    println!("Part one: {total_score_p1}");

    let guide_p2 = read_guide_p2("src/input.txt").unwrap();
    let total_score_p2: u64 = guide_p2.iter().map(|g| g.score() as u64).sum();
    println!("Part two: {total_score_p2}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{read_guide_p1, read_guide_p2};

    #[test]
    fn test_part_one() {
        assert_eq!(
            15,
            read_guide_p1("src/test_input.txt")
                .unwrap()
                .iter()
                .map(|g| g.score() as u64)
                .sum::<u64>()
        );
    }

    #[test]
    fn test_part_two() {
        assert_eq!(
            12,
            read_guide_p2("src/test_input.txt")
                .unwrap()
                .iter()
                .map(|g| g.score() as u64)
                .sum::<u64>()
        );
    }
}
