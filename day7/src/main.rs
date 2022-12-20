use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
struct File {
    size: usize,
    name: String,
}

#[derive(Debug, Clone)]
enum Command {
    Ls,
    Cd(String),
}

impl Command {
    fn try_from_line(line: &str) -> Result<Self> {
        let mut s = line.split(' ');
        let cmd = s
            .next()
            .ok_or_else(|| anyhow!("Line is over but expected command keyword"))?;
        match cmd {
            "ls" => Ok(Self::Ls),
            "cd" => {
                let target_dir = s
                    .next()
                    .ok_or_else(|| anyhow!("Line is over but expected `cd` target directory"))?;
                Ok(Self::Cd(target_dir.to_owned()))
            }
            c => Err(anyhow!("Unknown command encountered: {}", c)),
        }
    }
}

#[derive(Debug, Clone)]
enum Component {
    File(File),
    Directory(String),
}

impl Component {
    fn try_from_line(line: &str) -> Result<Self> {
        let mut s = line.split(' ');
        let first = s
            .next()
            .ok_or_else(|| anyhow!("Line is over but expected data"))?;
        match first {
            "dir" => {
                let dir_name = s
                    .next()
                    .ok_or_else(|| anyhow!("Line is over but expected directory name"))?;
                Ok(Self::Directory(dir_name.to_owned()))
            }
            f if f.is_ascii() => {
                let size = f
                    .parse::<usize>()
                    .map_err(|_| anyhow!("Expected integer file size, got: {}", f))?;
                let file_name = s
                    .next()
                    .ok_or_else(|| anyhow!("Line is over but expected file name"))?;
                Ok(Self::File(File {
                    size,
                    name: file_name.to_owned(),
                }))
            }
            c => Err(anyhow!("Unexepcted first element to data line: {}", c)),
        }
    }
}

#[derive(Debug, Clone)]
enum OutputLine {
    Command(Command),
    Component(Component),
    Eof,
}

#[derive(Debug, Clone)]
struct Directory {
    name: String,
    files: Option<HashMap<String, File>>,
    children: Option<HashMap<String, Rc<RefCell<Self>>>>,
    parent: Option<Weak<RefCell<Self>>>,
}

impl Directory {
    fn new(name: String) -> Self {
        Self {
            name,
            files: None,
            children: None,
            parent: None,
        }
    }
    fn add_child_dir(&mut self, name: String, new_dir: Rc<RefCell<Self>>) {
        if let Some(ch) = &mut self.children {
            ch.insert(name, new_dir);
        } else {
            self.children = Some(HashMap::from([(name, new_dir)]))
        }
    }
    fn add_file(&mut self, name: String, file: File) {
        if let Some(fs) = &mut self.files {
            fs.insert(name, file);
        } else {
            self.files = Some(HashMap::from([(name, file)]))
        }
    }
    fn set_parent(&mut self, parent: Weak<RefCell<Self>>) {
        self.parent = Some(parent);
    }
    fn print(&self, indent_level: usize) -> String {
        let mut dir_idn = String::new();
        for _ in 0..indent_level {
            dir_idn += " ";
        }
        let mut idn = dir_idn.clone();
        for _ in 0..2 {
            idn += " ";
        }
        let mut s = String::new();
        s += &format!("\n{}- {} (dir)\n", dir_idn, self.name);
        if let Some(f) = &self.files {
            s += &f
                .iter()
                .map(|(k, v)| format!("{}- {} (file, size={})", idn, k, v.size))
                .collect::<Vec<String>>()
                .join("\n");
        }
        if let Some(c) = &self.children {
            s += &c
                .iter()
                .map(|(_, v)| v.borrow().print(indent_level + 2))
                .collect::<Vec<String>>()
                .join("\n");
        }
        s
    }
    fn total_size(&self, dir_vec: &mut Vec<(String, usize)>) -> usize {
        let mut dir_size = 0usize;
        if let Some(f) = &self.files {
            dir_size += f.iter().map(|(_, v)| v.size).sum::<usize>();
        }
        if let Some(c) = &self.children {
            dir_size += c
                .iter()
                .map(|(_, v)| v.borrow().total_size(dir_vec))
                .sum::<usize>();
        }
        dir_vec.push((self.name.clone(), dir_size));
        dir_size
    }
}

fn parse_raw_output(input: &str) -> Result<Vec<OutputLine>> {
    let mut output_lines = Vec::new();
    for line in std::fs::read_to_string(input)?.lines() {
        if line.starts_with('$') {
            output_lines.push(OutputLine::Command(Command::try_from_line(&line[2..])?));
        } else {
            output_lines.push(OutputLine::Component(Component::try_from_line(line)?));
        }
    }
    output_lines.push(OutputLine::Eof);
    Ok(output_lines)
}

fn parse_intermediate_representation(ir: &[OutputLine]) -> Result<Rc<RefCell<Directory>>> {
    let ir_len = ir.len();
    let root = Rc::new(RefCell::new(Directory::new("/".to_owned())));
    let mut ir_cursor = 0;
    let mut current = Rc::clone(&root);
    while ir_cursor < ir_len {
        println!("Cursor: {:?}", &ir[ir_cursor]);
        match &ir[ir_cursor] {
            OutputLine::Command(cmd) => match cmd {
                Command::Cd(s) if matches!(s.as_str(), "/") => {
                    current = Rc::clone(&root);
                    ir_cursor += 1;
                }
                Command::Cd(s) if matches!(s.as_str(), "..") => {
                    let target;
                    if let Some(p) = &current.borrow().parent {
                        target = Rc::clone(
                            &p.upgrade()
                                .ok_or_else(|| anyhow!("Could not `upgrade` weak ref"))?,
                        );
                    } else {
                        return Err(anyhow!(
                            "Tried to access non-existent parent for directory {}",
                            current.borrow().name
                        ));
                    }
                    current = target;
                    ir_cursor += 1;
                }
                Command::Cd(s) => {
                    let target;
                    if let Some(dirs) = &current.borrow().children {
                        target = Rc::clone(dirs.get(s).ok_or_else(|| {
                            anyhow!("Tried to access non-existent directory: {}", &s)
                        })?);
                    } else {
                        return Err(anyhow!(
                            "No sub-directories associated with current directory: {}",
                            current.borrow().name
                        ));
                    }
                    let tmp = Weak::clone(&Rc::downgrade(&current));
                    current = target;
                    current.borrow_mut().set_parent(tmp);
                    ir_cursor += 1;
                }
                Command::Ls => loop {
                    if ir_cursor + 1 >= ir_len {
                        break;
                    }
                    match &ir[ir_cursor + 1] {
                        OutputLine::Component(Component::File(f)) => {
                            current.borrow_mut().add_file(f.name.clone(), f.clone());
                            ir_cursor += 1;
                        }
                        OutputLine::Component(Component::Directory(d)) => {
                            let new_dir = Directory::new(d.clone());
                            current
                                .borrow_mut()
                                .add_child_dir(d.clone(), Rc::new(RefCell::new(new_dir)));
                            ir_cursor += 1;
                        }
                        _ => {
                            ir_cursor += 1;
                            break;
                        }
                    }
                },
            },
            OutputLine::Eof => {
                break;
            }
            OutputLine::Component(_) => {
                return Err(anyhow!("Unexpected `Component` in outer loop"))
            }
        }
    }
    Ok(root)
}

fn main() {
    let terminal_output = parse_raw_output("src/input.txt").unwrap();
    let file_sys = parse_intermediate_representation(&terminal_output).unwrap();
    println!("{}", file_sys.borrow().print(0));
    let mut dir_size_vec = Vec::new();
    let root_size = file_sys.borrow().total_size(&mut dir_size_vec);
    let part_one_sum = dir_size_vec
        .iter()
        .filter_map(|&(_, s)| (s <= 100_000).then_some(s))
        .sum::<usize>();
    for dsv in dir_size_vec.iter() {
        println!("`{}`: {}", dsv.0, dsv.1);
    }
    println!("Part one: {part_one_sum}");

    let total_space = 70_000_000;
    let needed_space = 30_000_000;
    let unused_space = total_space - root_size;
    let target = needed_space - unused_space;
    let part_two_sum = dir_size_vec
        .iter()
        .filter_map(|&(_, s)| ((s as isize - target as isize) > 0).then_some(s))
        .min()
        .unwrap();
    println!("Part two: {part_two_sum}");
}

#[cfg(test)]
mod tests {
    use crate::{parse_intermediate_representation, parse_raw_output};

    #[test]
    fn test_part_one() {
        let file_sys =
            parse_intermediate_representation(&parse_raw_output("src/test_input.txt").unwrap())
                .unwrap();
        let mut dir_size_vec = Vec::new();
        let _root_size = file_sys.borrow().total_size(&mut dir_size_vec);
        let part_one_sum = dir_size_vec
            .iter()
            .filter_map(|&(_, s)| (s <= 100_000).then_some(s))
            .sum::<usize>();
        assert_eq!(95_437, part_one_sum);
    }

    #[test]
    fn test_part_two() {
        let file_sys =
            parse_intermediate_representation(&parse_raw_output("src/test_input.txt").unwrap())
                .unwrap();
        let mut dir_size_vec = Vec::new();
        let root_size = file_sys.borrow().total_size(&mut dir_size_vec);
        let total_space = 70_000_000;
        let needed_space = 30_000_000;
        let unused_space = total_space - root_size;
        let target = needed_space - unused_space;

        let part_two_sum = dir_size_vec
            .iter()
            .filter_map(|&(_, s)| ((s as isize - target as isize) > 0).then_some(s))
            .min()
            .unwrap();
        assert_eq!(24_933_642, part_two_sum);
    }
}
