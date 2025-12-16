use std::fs::File;
use std::io::{self, BufRead};

pub fn read_file_lines(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);

    let mut lines_vec = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        match line {
            Ok(content) => {
                if !content.trim().is_empty() {
                    lines_vec.push(content);
                }
            }
            Err(e) => eprintln!("Error at line {}: {}", index + 1, e),
        }
    }

    Ok(lines_vec)
}
