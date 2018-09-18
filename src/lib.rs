extern crate nom_sql;

use nom_sql::{
    InsertStatement,
    Literal,
    parser,
    parser::SqlQuery,
    SqlQuery::Insert,
    Table,
};
use std::io::BufRead;
use std::io::Error;
use std::iter::empty;


fn extract_data(query: SqlQuery) -> Result<Vec<Vec<Literal>>, String> {
    match query {
        Insert(InsertStatement {
                   table: Table { name, .. },
                   data, ..
               }) => {
            if name == "externallinks" {
                Ok(data)
            } else {
                Err(format!("Wrong table: '{}'", name))
            }
        }
        parsed => {
            Err(format!("Not an import statement: {:?}", parsed))
        }
    }
}

fn extract_target_string(mut values: Vec<Literal>, target: usize) -> Result<String, String> {
    if values.len() <= target {
        Err(format!("Too few inserted values: {:?}", values))
    } else {
        match values.swap_remove(target) {
            Literal::String(s) => Ok(s),
            non_string_val => {
                Err(format!("Invalid inserted value type: {:?} (at index {})", non_string_val, target))
            }
        }
    }
}

fn single_error_iterator<T: 'static>(s: String) -> Box<Iterator<Item=Result<T, String>>> {
    Box::new(std::iter::once(Err(s)))
}

fn extract_urls_from_statement(input: SqlQuery) -> Box<Iterator<Item=Result<String, String>>> {
    let target_index = 2;
    match extract_data(input) {
        Ok(insert_data) => Box::new(
            insert_data.into_iter()
                .map(move |v| extract_target_string(v, target_index))
        ),
        Err(s) => single_error_iterator(s)
    }
}

fn is_comment(line_bytes: &Vec<u8>) -> bool {
    line_bytes.starts_with(b"--") ||
        line_bytes.starts_with(b"/*") ||
        line_bytes.is_empty()
}

fn is_complete_statement(statement: &Vec<u8>) -> bool {
    statement.ends_with(b";")
}

#[derive(Debug)]
struct ScanState {
    current_statement: Vec<u8>,
    target_field: Option<usize>,
}

impl ScanState {
    fn new() -> ScanState {
        ScanState {
            current_statement: Vec::with_capacity(1024),
            target_field: None,
        }
    }

    fn add_line(&mut self, line_bytes: &mut Vec<u8>) -> Option<Result<SqlQuery, &'static str>> {
        if is_comment(line_bytes) { None } else {
            self.current_statement.append(line_bytes);
            if is_complete_statement(&self.current_statement) {
                let parsed_sql = parser::parse_query_bytes(&self.current_statement);
                self.current_statement.clear();
                Some(parsed_sql)
            } else { None }
        }
    }
}

fn scan_binary_lines(
    scan_state: &mut ScanState,
    mut line_result: Result<Vec<u8>, Error>,
) -> Option<Box<Iterator<Item=Result<String, String>>>> {
    match line_result {
        Ok(ref mut line_bytes) => {
            match scan_state.add_line(line_bytes) {
                Some(Ok(statement)) => Some(extract_urls_from_statement(statement)),
                Some(Err(s)) => Some(single_error_iterator(format!("Unable to parse sql: {}", s))),
                None => Some(Box::new(empty()))
            }
        }
        Err(err) => Some(single_error_iterator(format!("Unable to read line: {}", err)))
    }
}

pub fn iter_string_urls<T: BufRead>(input: T) -> impl Iterator<Item=Result<String, String>> {
    input.split(b'\n')
        .scan(ScanState::new(), scan_binary_lines)
        .flatten()
}