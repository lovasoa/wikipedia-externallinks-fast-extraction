extern crate nom_sql;

use nom_sql::{
    InsertStatement,
    Literal,
    parser,
    parser::SqlQuery,
    SqlQuery::Insert,
    Table,
};
use std::io::{self, BufRead};


fn extract_data(query: SqlQuery) -> Option<Vec<Vec<Literal>>> {
    match query {
        Insert(InsertStatement {
                   table: Table { name, .. },
                   data, ..
               }) => if name == "externallinks" { Some(data) } else { None },
        parsed => {
            eprintln!("Not an import statement: {:?}", parsed);
            None
        }
    }
}

fn extract_target_string(mut values: Vec<Literal>, target: usize) -> Option<String> {
    if values.len() <= target {
        eprintln!("Too few inserted values: {:?}", values);
        None
    } else {
        match values.swap_remove(target) {
            Literal::String(s) => Some(s),
            non_string_val => {
                eprintln!("Invalid inserted value type: {:?} (at index {})", non_string_val, target);
                None
            }
        }
    }
}

fn extract_urls_from_statement(input: SqlQuery) -> impl Iterator<Item=String> {
    let target_index = 2;
    extract_data(input).into_iter()
        .flat_map(|data| data.into_iter())
        .filter_map(move |v| extract_target_string(v, target_index))
}

fn print_urls_from_statement(statement_bytes: &Vec<u8>) {
    if let Ok(query) = parser::parse_query_bytes(statement_bytes) {
        for url in extract_urls_from_statement(query) {
            println!("{}", url);
        }
    } else {
        eprintln!("Unable to parse the sql statement.");
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
    let mut current_statement: Vec<u8> = Vec::with_capacity(1024);

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
