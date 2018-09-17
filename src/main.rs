extern crate nom_sql;
extern crate encoding;

use std::io::{self, BufRead};
use nom_sql::{
    parser::parse_query,
    SqlQuery::Insert,
    InsertStatement,
    Table,
    Literal,
};

use encoding::{Encoding, DecoderTrap};
use encoding::all::UTF_8;

fn extract_data(input: &str) -> Option<Vec<Vec<Literal>>> {
    match parse_query(input) {
        Ok(Insert(InsertStatement {
                      table: Table { name, .. },
                      data, ..
                  })) => if name == "externallinks" { Some(data) } else { None },
        parsed => {
            eprintln!("Not a valid import statement: {} ({:?})", input, parsed);
            None
        }
    }
}

fn extract_urls_from_statement(input: &str) -> impl Iterator<Item=String> {
    let target_index = 2;
    extract_data(input).into_iter()
        .flat_map(|data| data.into_iter())
        .filter(move |v| v.len() >= target_index + 1)
        .filter_map(move |mut v| match v.swap_remove(target_index) {
            Literal::String(s) => Some(s),
            _ => None
        })
}

fn print_urls_from_statement(statement_bytes: &Vec<u8>) {
    if let Ok(statement_str) = UTF_8.decode(statement_bytes, DecoderTrap::Replace) {
        for url in extract_urls_from_statement(&statement_str) {
            println!("{}", url);
        }
    } else {
        eprintln!("Unable to decode the line (should never happen).");
    }
}

fn is_comment(line_bytes: &Vec<u8>) -> bool {
    line_bytes.starts_with(b"--") ||
        line_bytes.starts_with(b"/*")
}

fn is_complete_statement(statement: &Vec<u8>) -> bool {
    statement.ends_with(b";")
}

fn main() {
    let stdin = io::stdin();
    let mut current_statement : Vec<u8> = Vec::with_capacity(1024);

    for mut line_result in stdin.lock().split(b'\n') {
        match line_result {
            Ok(ref mut line_bytes) => {
                if !is_comment(line_bytes) {
                    current_statement.append(line_bytes);
                    if is_complete_statement(&current_statement) {
                        print_urls_from_statement(&current_statement);
                        current_statement.clear();
                    }
                }
            }
            Err(err) => {
                eprintln!("Unable to read line: {}", err);
            }
        }
    }
}
