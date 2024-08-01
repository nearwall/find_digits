#![warn(clippy::cargo, clippy::pedantic)]
#![allow(
    clippy::cargo_common_metadata,
    clippy::cast_precision_loss,
    clippy::multiple_crate_versions
)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::exit,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use clap::Parser;

const REPORT_DELAY: std::time::Duration = std::time::Duration::from_secs(10);

static LETTERS_DIGITS: [(&str, &str, char); 9] = [
    ("one", "one", '1'),
    ("two", "two", '2'),
    ("six", "six", '6'),
    ("fou", "four", '4'),
    ("fiv", "five", '5'),
    ("nin", "nine", '9'),
    ("sev", "seven", '7'),
    ("eig", "eight", '8'),
    ("thr", "three", '3'),
];

static LETTERS_DIGITS_REV: [(&str, &str, char); 9] = [
    ("one", "one", '1'),
    ("two", "two", '2'),
    ("six", "six", '6'),
    ("our", "four", '4'),
    ("ive", "five", '5'),
    ("ine", "nine", '9'),
    ("ven", "seven", '7'),
    ("ght", "eight", '8'),
    ("ree", "three", '3'),
];

const LETTERS_DIGIT_MIN_LEN: usize = 3;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();

    let reader = match File::open(&args.file) {
        Ok(f) => BufReader::new(f),
        Err(e) => {
            println!("Fail to open file {}: {e:?}", &args.file);
            exit(1);
        },
    };

    let mut timestamp = Instant::now();
    let start_timestamp = Instant::now();

    let mut total_sum = 0_u32;
    let mut parsed_lines = 0_u32;
    let mut incorrect_lines = 0_u32;

    for (number, read_result) in reader.lines().enumerate() {
        parsed_lines += 1;

        let line = match read_result {
            Ok(l) => l,
            Err(e) => {
                println!("File {} broken line(number {number}): {e:?}", &args.file);
                continue;
            },
        };

        if line.is_empty() {
            incorrect_lines += 1;
            continue;
        }

        match extract_number(&line) {
            Some(amount) => {
                total_sum += amount;
            },
            None => {
                incorrect_lines += 1;
            },
        }

        if timestamp.elapsed() > REPORT_DELAY {
            timestamp = Instant::now();
            println!(
                "{:?} Parsed lines: {parsed_lines}, Incorrect lines {incorrect_lines}, Total amount: {total_sum}",
                SystemTime::now().duration_since(UNIX_EPOCH)
            );
        }
    }

    println!(
        "{:?} Parsed lines: {parsed_lines}, Incorrect lines {incorrect_lines}, Total amount: {total_sum}, Elapsed {:?}",
        SystemTime::now().duration_since(UNIX_EPOCH),
        start_timestamp.elapsed()
    );

    println!("\nTotal amount: {total_sum}");
}

fn extract_number(line: &str) -> Option<u32> {
    let res = find(line);
    let r_res = r_find(line, res.last_parsed_position);

    let (Some(fst), Some(lst)) = (res.number, r_res.number) else {
        return None;
    };

    format!("{fst}{lst}").parse().ok()
}

#[derive(Debug, PartialEq, Eq)]
struct SearchResult {
    number: Option<char>,
    last_parsed_position: usize,
}

fn find(line: &str) -> SearchResult {
    let line_length = line.len();
    let mut pos = 0;

    let mut character = line.chars();

    while line_length > pos {
        if let Some(c) = character.next() {
            if c.is_ascii_digit() {
                return SearchResult {
                    number: Some(c),
                    last_parsed_position: pos,
                };
            }
        }

        let rest = line_length - pos;
        if rest < LETTERS_DIGIT_MIN_LEN {
            pos += 1;
            continue;
        }

        for i in LETTERS_DIGITS {
            if line[pos..pos + LETTERS_DIGIT_MIN_LEN] == *i.0 {
                let length = i.1.len();
                if rest >= length && line[pos..pos + length] == *i.1 {
                    return SearchResult {
                        number: Some(i.2),
                        last_parsed_position: pos,
                    };
                }

                // it's correct while all beginnings of digits are unique
                break;
            }
        }

        pos += 1;
    }

    SearchResult {
        number: None,
        last_parsed_position: line_length,
    }
}

fn r_find(line: &str, found_pos: usize) -> SearchResult {
    let line_length = line.len();
    let mut pos = line_length;

    let mut character = line.chars().rev();

    while pos > found_pos {
        if let Some(c) = character.next() {
            if c.is_ascii_digit() {
                return SearchResult {
                    number: Some(c),
                    last_parsed_position: pos,
                };
            }
        }

        if pos < LETTERS_DIGIT_MIN_LEN {
            pos -= 1;
            continue;
        }

        for i in LETTERS_DIGITS_REV {
            if line[pos - LETTERS_DIGIT_MIN_LEN..pos] == *i.0 {
                let length = i.1.len();
                if pos >= length && line[pos - length..pos] == *i.1 {
                    return SearchResult {
                        number: Some(i.2),
                        last_parsed_position: pos - length,
                    };
                }

                // it's correct while all endings of digits are unique
                break;
            }
        }

        pos -= 1;
    }

    SearchResult {
        number: None,
        last_parsed_position: pos,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find() {
        let strings = [
            "eightwothree",
            "abcone2threexyz",
            "treb7uchet",
            "7pqrstsixteen",
            "abcdefg",
        ];
        let expected = [
            SearchResult {
                number: Some('8'),
                last_parsed_position: 0,
            },
            SearchResult {
                number: Some('1'),
                last_parsed_position: 3,
            },
            SearchResult {
                number: Some('7'),
                last_parsed_position: 4,
            },
            SearchResult {
                number: Some('7'),
                last_parsed_position: 0,
            },
            SearchResult {
                number: None,
                last_parsed_position: 7,
            },
        ];

        for (pos, line) in strings.into_iter().enumerate() {
            let res = find(line);

            assert_eq!(res, expected[pos]);
        }
    }

    #[test]
    fn test_r_find() {
        let strings = [
            ("eightwothree", 0),
            ("abcone2threexyz", 3),
            ("treb7uchet", 4),
            ("7pqrstsixteen", 0),
            ("abcdefg", 7),
            ("abcdefg", 0),
        ];
        let expected = [
            SearchResult {
                number: Some('3'),
                last_parsed_position: 7,
            },
            SearchResult {
                number: Some('3'),
                last_parsed_position: 7,
            },
            SearchResult {
                number: Some('7'),
                last_parsed_position: 5,
            },
            SearchResult {
                number: Some('6'),
                last_parsed_position: 6,
            },
            SearchResult {
                number: None,
                // because 7 it's the smallest number by condition in previous array
                last_parsed_position: 7,
            },
            SearchResult {
                number: None,
                last_parsed_position: 0,
            },
        ];

        for (pos, (line, found_pos)) in strings.into_iter().enumerate() {
            let res = r_find(line, found_pos);

            assert_eq!(res, expected[pos]);
        }
    }

    #[test]
    fn test_extract_number() {
        let strings = [
            "eightwothree",
            "abcone2threexyz",
            "treb7uchet",
            "7pqrstsixteen",
            "abcdefg",
        ];
        let expected = [Some(83), Some(13), Some(77), Some(76), None];

        for (pos, line) in strings.into_iter().enumerate() {
            let result = extract_number(line);
            assert_eq!(result, expected[pos])
        }
    }
}
