use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Noop,
    Addx(isize),
}

impl Instruction {
    fn num_cycles(&self) -> usize {
        match self {
            Self::Noop => 1,
            Self::Addx(_) => 2,
        }
    }

    fn try_from_line(line: &str) -> Result<Self> {
        let mut s = line.split(' ');
        match s
            .next()
            .ok_or_else(|| anyhow!("Line over but expected instruction"))?
        {
            "addx" => {
                let add_arg = s
                    .next()
                    .ok_or_else(|| anyhow!("Line over but expected argument for `addx`"))?;
                Ok(Self::Addx(add_arg.parse::<isize>()?))
            }
            "noop" => Ok(Self::Noop),
            c => Err(anyhow!("Unsupported command: {}", c)),
        }
    }
}

struct VM {
    reg: isize,
    current_cycle: usize,
    history: Vec<(isize, usize)>,
}

impl VM {
    fn new() -> Self {
        Self {
            reg: 1,
            current_cycle: 1,
            history: vec![(1, 1)],
        }
    }
    fn exe_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Noop => {
                for _ in 0..instruction.num_cycles() {
                    self.cycle();
                }
            }
            Instruction::Addx(arg) => {
                for _ in 0..instruction.num_cycles() - 1 {
                    self.cycle();
                }
                self.reg += arg;
                self.cycle();
            }
        }
    }
    fn cycle(&mut self) {
        self.current_cycle += 1;
        self.history.push((self.reg, self.current_cycle));
    }
}

fn read_program(input: &str) -> Result<Vec<Instruction>> {
    let mut program = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        program.push(Instruction::try_from_line(line)?);
    }
    Ok(program)
}

fn draw_crt(history: &[(isize, usize)], crt: &mut [[char; 40]; 6]) {
    let mut cycle_idx = 0;
    for row in crt.iter_mut() {
        for (col_idx, col) in row.iter_mut().enumerate() {
            let sprite_loc = history[cycle_idx].0;
            let c = if (col_idx as isize - sprite_loc).abs() <= 1 {
                '#'
            } else {
                '.'
            };
            *col = c;
            cycle_idx += 1;
        }
    }
}

fn main() {
    let mut vm = VM::new();
    for instruction in read_program("src/input.txt").unwrap() {
        vm.exe_instruction(instruction);
    }
    let mut ss_sum = 0;
    let mut ss_idx = 19;
    for _ in 0..6 {
        let (r, c) = vm.history[ss_idx];
        ss_sum += c as isize * r;
        ss_idx += 40;
    }
    println!("Part one: {ss_sum}");

    let mut crt = [['.'; 40]; 6];
    draw_crt(&vm.history, &mut crt);

    println!("Part two:\n");
    for row in crt.iter() {
        for c in row.iter() {
            print!("{}", c);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use crate::{read_program, VM};

    #[test]
    fn test_part_one() {
        let mut vm = VM::new();
        for instruction in read_program("src/test_input.txt").unwrap() {
            vm.exe_instruction(instruction);
        }
        let mut ss_sum = 0;
        let mut ss_idx = 19;
        for _ in 0..6 {
            let (r, c) = vm.history[ss_idx];
            println!("{}-th cycle reg: {}, {}", ss_idx, c, r);
            ss_sum += c as isize * r;
            ss_idx += 40;
        }
        assert_eq!(13_140, ss_sum);
    }
}
