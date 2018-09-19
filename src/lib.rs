extern crate nom_sql;

use nom_sql::{
    InsertStatement,
    Literal,
    parser,
    parser::SqlQuery,
    SqlQuery::Insert,
    SqlQuery::CreateTable,
    Table,
};
use std::io::BufRead;
use std::io::Error;
use std::iter::empty;
use ::ExtractedSql::InsertData;
use nom_sql::CreateTableStatement;
use nom_sql::ColumnSpecification;
use nom_sql::Column;

struct TargetColumn<'a> {
    name: &'a str,
    table: &'a str
}

const TARGET_COLUMN: TargetColumn<'static> = TargetColumn {
    name: "el_to",
    table: "externallinks"
};

enum ExtractedSql {
    InsertData(Vec<Vec<Literal>>),
    CreateTableData(usize),
    Error(String),
}

fn extract_data(query: SqlQuery) -> ExtractedSql {
    match query {
        Insert(InsertStatement {
                   table: Table { name, .. },
                   data, ..
               }) => {
            if name == TARGET_COLUMN.table {
                InsertData(data)
            } else {
                ExtractedSql::Error(format!("Wrong table: '{}'", name))
            }
        },
        CreateTable(CreateTableStatement {
                        table: Table { name, .. },
                        fields,
                        ..
                    }) => {
            if name == TARGET_COLUMN.table {
                match find_target_field_index(fields) {
                    Some(i) => ExtractedSql::CreateTableData(i),
                    None => ExtractedSql::Error(format!("Target field not found"))
                }
            } else {
                ExtractedSql::Error(format!("Wrong table: '{}'", name))
            }
        }
        parsed => {
            ExtractedSql::Error(format!("Not an import statement: {:?}", parsed))
        }
    }
}

fn find_target_field_index(fields: Vec<ColumnSpecification>) -> Option<usize> {
    fields.iter()
        .position(|spec| {
            let ColumnSpecification {
                column: Column { name, .. },
                ..
            } = spec;
            name == TARGET_COLUMN.name
        })
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

fn extract_urls_from_insert_data(
    data: Vec<Vec<Literal>>,
    target_index: usize,
) -> impl Iterator<Item=Result<String, String>> {
    data.into_iter()
        .map(move |v| extract_target_string(v, target_index))
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

enum ScanLineAction {
    Pass,
    ReportError(String),
    ExtractFrom(Vec<Vec<Literal>>, usize),
}

impl ScanState {
    fn new() -> ScanState {
        ScanState {
            current_statement: Vec::with_capacity(1024),
            target_field: None,
        }
    }

    fn add_line(&mut self, line_bytes: &mut Vec<u8>) -> ScanLineAction {
        if is_comment(line_bytes) {
            ScanLineAction::Pass
        } else {
            self.current_statement.append(line_bytes);
            if is_complete_statement(&self.current_statement) {
                let parsed_sql = parser::parse_query_bytes(&self.current_statement);
                self.current_statement.clear();
                match parsed_sql {
                    Ok(sql) => match extract_data(sql) {
                        InsertData(data) => {
                            if let Some(i) = self.target_field {
                                ScanLineAction::ExtractFrom(data, i)
                            } else {
                                ScanLineAction::ReportError("Insert statement before create table".into())
                            }
                        },
                        ExtractedSql::CreateTableData(index) => {
                            self.target_field = Some(index);
                            ScanLineAction::Pass
                        },
                        ExtractedSql::Error(err) => ScanLineAction::ReportError(err),
                    },
                    Err(s) => ScanLineAction::ReportError(s.to_string())
                }
            } else { ScanLineAction::Pass }
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
                ScanLineAction::ExtractFrom(data, i) => Some(Box::new(extract_urls_from_insert_data(data, i))),
                ScanLineAction::ReportError(s) => Some(single_error_iterator(s)),
                ScanLineAction::Pass => Some(Box::new(empty()))
            }
        }
        Err(err) => Some(single_error_iterator(format!("Unable to read line: {}", err)))
    }
}

pub fn iter_string_urls<T: BufRead>(input: T) -> impl Iterator<Item=Result<String, String>> {
    input.split(b'\n')
        .scan(ScanState::new(), scan_binary_lines)
        .flat_map(|urls| urls)
}
