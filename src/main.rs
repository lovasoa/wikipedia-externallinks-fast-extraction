extern crate nom_sql;

use std::io::{self, BufRead};
use nom_sql::{
    parser::parse_query,
    SqlQuery::Insert,
    InsertStatement,
    Table,
    Literal
};

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

fn process_line(input: &str) -> Vec<String> {
    extract_data(input).iter()
        .flat_map(|data| data.iter())
        .flat_map(|v| match v.get(2) {
            Some(Literal::String(s)) => Some(s.clone()),
            _ => None
        })
        .collect()
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(ref line_str) => {
                for url in process_line(line_str) {
                    println!("{}", url);
                }
            },
            Err(err) => {
                eprintln!("Unable to read line: {}", err);
            }
        }
    }
}
