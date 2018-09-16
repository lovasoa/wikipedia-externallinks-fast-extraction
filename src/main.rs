extern crate nom_sql;
extern crate encoding;

use std::io::{self, BufRead};
use nom_sql::{
    parser::parse_query,
    SqlQuery::Insert,
    InsertStatement,
    Table,
    Literal
};

use encoding::{Encoding, DecoderTrap};
use encoding::all::UTF_8;

fn extract_data(input: &str) -> Option<Vec<Vec<Literal>>> {
    match parse_query(input) {
        Ok(Insert(InsertStatement {
                table: Table {name, ..},
                data, ..
        })) => if name == "externallinks" {Some(data)} else {None},
        parsed => {
            eprintln!("Not a valid import statement: {} ({:?})", input, parsed);
            None
        }
    }
}

fn process_line(input: &str) -> impl Iterator<Item=String> {
    let target_index = 2;
    extract_data(input).into_iter()
        .flat_map(|data| data.into_iter())
        .filter(move |v| v.len() >= target_index + 1)
        .flat_map(move |mut v| match v.swap_remove(target_index) {
            Literal::String(s) => Some(s),
            _ => None
        })
}

fn main() {
    let stdin = io::stdin();
    for line_result in stdin.lock().split(b'\n') {
        match line_result {
            Ok(ref line_bytes) => {
                if let Ok(line_str) = UTF_8.decode(line_bytes, DecoderTrap::Replace) {
                    for url in process_line(&line_str) {
                        println!("{}", url);
                    }
                } else {
                    eprintln!("Unable to decode the line (should never happen).");
                }
            },
            Err(err) => {
                eprintln!("Unable to read line: {}", err);
            }
        }
    }
}
