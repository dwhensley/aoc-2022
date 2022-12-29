use std::cmp::Ordering;

use anyhow::{anyhow, Error, Result};

#[derive(Debug, Clone)]
struct RawPacketPair {
    left: String,
    right: String,
}

fn read_line_pairs(input: &str) -> Result<Vec<RawPacketPair>> {
    let mut pairs = Vec::new();
    for pair in std::fs::read_to_string(input)?.split("\n\n") {
        let p = pair.lines().map(|l| l.to_owned()).collect::<Vec<String>>();
        if p.len() > 2 {
            return Err(anyhow!("Expected raw packet pair, got {} lines", p.len()));
        }
        let mut p_iter = p.into_iter();
        let left = p_iter
            .next()
            .ok_or_else(|| anyhow!("Expected left packet line"))?;
        let right = p_iter
            .next()
            .ok_or_else(|| anyhow!("Expected right packet line"))?;
        pairs.push(RawPacketPair { left, right });
    }
    Ok(pairs)
}

fn read_packets(input: &str) -> Result<Vec<Packet>> {
    let mut packets = Vec::new();
    for line in std::fs::read_to_string(input)?
        .split("\n\n")
        .flat_map(|p| p.lines())
    {
        packets.push(Packet::try_from(line)?);
    }
    Ok(packets)
}

#[derive(Debug, Copy, Clone)]
enum Token {
    LBracket,
    RBracket,
    Comma,
    Uint(usize),
}

struct Lexer<'s> {
    line: &'s [char],
    tokens: Vec<Token>,
    start: usize,
    current: usize,
}

impl<'s> Lexer<'s> {
    fn new(line: &'s [char]) -> Self {
        Self {
            line,
            tokens: Vec::new(),
            start: 0,
            current: 0,
        }
    }

    fn lex_tokens(mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.lex_token()?;
        }
        Ok(self.tokens)
    }

    fn lex_token(&mut self) -> Result<()> {
        let c = self.advance();
        match c {
            '[' => self.tokens.push(Token::LBracket),
            ']' => self.tokens.push(Token::RBracket),
            ',' => self.tokens.push(Token::Comma),
            c if c.is_ascii_digit() => self.integer()?,
            c => return Err(anyhow!("Unexpected token: {}", c)),
        }
        Ok(())
    }

    fn integer(&mut self) -> Result<()> {
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                let _ = self.advance();
            } else {
                break;
            }
        }
        let integer =
            String::from_iter(self.line[self.start..self.current].iter()).parse::<usize>()?;
        self.tokens.push(Token::Uint(integer));
        Ok(())
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.line[self.current])
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.line.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.line[self.current - 1]
    }
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(mut self) -> Result<Packet> {
        if let Token::LBracket = self.advance() {
            Ok(Packet::List(self.parse_list()?))
        } else {
            return Err(anyhow!("Expected outermost list"));
        }
    }

    fn parse_list(&mut self) -> Result<Vec<Packet>> {
        let mut list = Vec::new();
        let mut comma_expected = false;
        while self.peek().is_some() {
            let t = self.advance();
            match t {
                Token::RBracket => {
                    return Ok(list);
                }
                Token::LBracket if !comma_expected => {
                    list.push(Packet::List(self.parse_list()?));
                    comma_expected = true;
                }
                Token::LBracket if comma_expected => {
                    return Err(anyhow!("Unexpected (no separating comma)"));
                }
                Token::Comma if comma_expected => {
                    comma_expected = false;
                }
                Token::Comma if !comma_expected => {
                    return Err(anyhow!("Unexpected comma!"));
                }
                Token::Uint(v) if !comma_expected => {
                    list.push(Packet::Uint(v));
                    comma_expected = true;
                    continue;
                }
                Token::Uint(v) if comma_expected => {
                    return Err(anyhow!("Unexpected integer: {}", v));
                }
                t => {
                    return Err(anyhow!("Unexpected token {:?} during parsing", t));
                }
            }
        }
        return Err(anyhow!("No packet to parse!"));
    }

    fn peek(&self) -> Option<Token> {
        if self.is_at_end() {
            None
        } else {
            Some(self.tokens[self.current])
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn advance(&mut self) -> Token {
        self.current += 1;
        self.tokens[self.current - 1]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Packet {
    Uint(usize),
    List(Vec<Self>),
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Packet::Uint(l), Packet::Uint(r)) => l.cmp(r),
            (Packet::List(l), Packet::List(r)) => {
                for (l, r) in l.iter().zip(r.iter()) {
                    match l.cmp(r) {
                        Ordering::Less => return Ordering::Less,
                        Ordering::Equal => {}
                        Ordering::Greater => return Ordering::Greater,
                    }
                }
                l.len().cmp(&r.len())
            }
            (Packet::Uint(_), Packet::List(_)) => {
                Packet::cmp(&Packet::List(vec![self.clone()]), other)
            }
            (Packet::List(_), Packet::Uint(_)) => {
                Packet::cmp(self, &Packet::List(vec![other.clone()]))
            }
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<&str> for Packet {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let chars = value.chars().collect::<Vec<char>>();
        Parser::new(Lexer::new(&chars).lex_tokens()?).parse()
    }
}

#[derive(Debug, Clone)]
struct PacketPair {
    left: Packet,
    right: Packet,
}

impl TryFrom<RawPacketPair> for PacketPair {
    type Error = Error;
    fn try_from(value: RawPacketPair) -> Result<Self, Self::Error> {
        let left_chars = value.left.chars().collect::<Vec<char>>();
        let right_chars = value.right.chars().collect::<Vec<char>>();
        let left = Parser::new(Lexer::new(&left_chars).lex_tokens()?).parse()?;
        let right = Parser::new(Lexer::new(&right_chars).lex_tokens()?).parse()?;
        Ok(Self { left, right })
    }
}
fn main() -> Result<()> {
    let mut pairs: Vec<PacketPair> = Vec::new();
    for p in read_line_pairs("src/input.txt")?.into_iter() {
        pairs.push(p.try_into()?);
    }
    let idx_sum = pairs
        .iter()
        .enumerate()
        .filter_map(|(idx, pp)| (pp.left < pp.right).then_some(idx + 1))
        .sum::<usize>();
    println!("Part one: {idx_sum}");

    let mut packets = read_packets("src/input.txt")?;
    let div_pack1 = Packet::List(vec![Packet::List(vec![Packet::Uint(2)])]);
    let div_pack2 = Packet::List(vec![Packet::List(vec![Packet::Uint(6)])]);
    packets.push(div_pack1.clone());
    packets.push(div_pack2.clone());
    packets.sort_unstable();
    let mut div_p1_idx = 0;
    let mut div_p2_idx = 0;
    for (idx, p) in packets.into_iter().enumerate() {
        if p == div_pack1 {
            div_p1_idx = idx + 1;
        }
        if p == div_pack2 {
            div_p2_idx = idx + 1;
            break;
        }
    }
    println!("Part two: {}", div_p1_idx * div_p2_idx);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{read_line_pairs, read_packets, Packet, PacketPair};

    #[test]
    fn test_part_one() {
        let mut pairs: Vec<PacketPair> = Vec::new();
        for p in read_line_pairs("src/test_input.txt").unwrap().into_iter() {
            pairs.push(p.try_into().unwrap());
        }
        let idx_sum = pairs
            .iter()
            .enumerate()
            .filter_map(|(idx, pp)| (pp.left < pp.right).then_some(idx + 1))
            .sum::<usize>();
        assert_eq!(13, idx_sum);
    }

    #[test]
    fn test_part_two() {
        let mut packets = read_packets("src/test_input.txt").unwrap();
        let div_pack1 = Packet::List(vec![Packet::List(vec![Packet::Uint(2)])]);
        let div_pack2 = Packet::List(vec![Packet::List(vec![Packet::Uint(6)])]);
        packets.push(div_pack1.clone());
        packets.push(div_pack2.clone());
        packets.sort_unstable();
        let mut div_p1_idx = 0;
        let mut div_p2_idx = 0;
        for (idx, p) in packets.into_iter().enumerate() {
            if p == div_pack1 {
                div_p1_idx = idx + 1;
            }
            if p == div_pack2 {
                div_p2_idx = idx + 1;
                break;
            }
        }
        assert_eq!(140, div_p1_idx * div_p2_idx);
    }
}
