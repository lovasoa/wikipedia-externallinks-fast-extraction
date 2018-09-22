extern crate nom_sql;
extern crate rayon;

use ::ExtractedSql::InsertData;
use nom_sql::{
    InsertStatement,
    Literal,
    parser,
    parser::SqlQuery,
    SqlQuery::CreateTable,
    SqlQuery::Insert,
    Table,
};
use nom_sql::Column;
use nom_sql::ColumnSpecification;
use nom_sql::CreateTableStatement;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::io;
use std::io::BufRead;
use std::sync::mpsc::channel;

struct TargetColumn<'a> {
    name: &'a str,
    table: &'a str,
}

const TARGET_COLUMN: TargetColumn<'static> = TargetColumn {
    name: "el_to",
    table: "externallinks",
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
        }
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

pub fn extract_urls_from_insert_data(
    data: Vec<Vec<Literal>>,
    target_index: usize,
) -> impl ParallelIterator<Item=Result<String, String>> {
    data.into_par_iter()
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

impl ScanState {
    fn new() -> ScanState {
        ScanState {
            current_statement: Vec::with_capacity(1_000_000),
            target_field: None,
        }
    }

    fn add_line(&mut self, line_bytes: &mut Vec<u8>) -> ScanLineAction {
        if is_comment(line_bytes) {
            ScanLineAction::Pass
        } else {
            self.current_statement.append(line_bytes);
            if is_complete_statement(&self.current_statement) {
                let scan_result = self.extract_scan_result();
                self.current_statement.clear();
                scan_result
            } else { ScanLineAction::Pass }
        }
    }

    fn extract_scan_result(&mut self) -> ScanLineAction {
        if let Some(target) = self.target_field {
            ScanLineAction::ExtractFrom(self.current_statement.clone(), target)
        } else {
            match parser::parse_query_bytes(&self.current_statement) {
                Ok(sql) => match extract_data(sql) {
                    InsertData(_) => {
                        ScanLineAction::ReportError("Insert statement before create table".into())
                    }
                    ExtractedSql::CreateTableData(index) => {
                        self.target_field = Some(index);
                        ScanLineAction::Pass
                    }
                    ExtractedSql::Error(err) => ScanLineAction::ReportError(err),
                },
                Err(s) => {
                    let source_sql: String = std::str::from_utf8(&self.current_statement)
                        .unwrap_or("invalid utf8")
                        .chars()
                        .take(150)
                        .chain(" [...]".chars())
                        .collect();
                    let err_string = format!("{} (while parsing: {})", s, source_sql);
                    ScanLineAction::ReportError(err_string)
                }
            }
        }
    }
}

enum ScanLineAction {
    Pass,
    ReportError(String),
    ExtractFrom(Vec<u8>, usize),
}

impl ScanLineAction {
    fn into_par_iter(self) -> impl ParallelIterator<Item=Result<String, String>> {
        let mut res = None;
        let mut err = None;
        match self {
            ScanLineAction::ExtractFrom(bytes, target) => {
                match parse_insert(&bytes) {
                    Ok(data) => { res = Some(extract_urls_from_insert_data(data, target)) }
                    Err(s) => { err = Some(s) }
                }
            }
            ScanLineAction::ReportError(s) => { err = Some(s) }
            ScanLineAction::Pass => (),
        };
        res.into_par_iter()
            .flatten()
            .chain(err.into_par_iter().map(|s| Err(s)))
    }
}

fn parse_insert(sql_statement: &Vec<u8>)
                -> Result<Vec<Vec<Literal>>, String> {
    match parser::parse_query_bytes(sql_statement) {
        Ok(sql) =>
            match extract_data(sql) {
                InsertData(data) => Ok(data),
                ExtractedSql::CreateTableData(_) => Err(format!("Unexpected CREATE TABLE")),
                ExtractedSql::Error(err) => Err(err),
            },
        Err(err) => Err(err.to_string())
    }
}

fn scan_binary_lines(
    scan_state: &mut ScanState,
    mut line_result: Result<Vec<u8>, io::Error>,
) -> Option<ScanLineAction> {
    let action = match line_result {
        Ok(ref mut line_bytes) => scan_state.add_line(line_bytes),
        Err(err) => ScanLineAction::ReportError(format!("Unable to read line: {}", err))
    };
    Some(action)
}

pub fn iter_string_urls<T: BufRead>(input: T)
                                    -> impl ParallelIterator<Item=Result<String, String>> {
    let rx = {
        let (tx, rx) = channel();

        input.split(b'\n')
            .scan(ScanState::new(), scan_binary_lines)
            .for_each(|x| tx.send(x).unwrap());

        rx
    };

    rx.into_iter()
        .par_bridge()
        .flat_map(ScanLineAction::into_par_iter)
}
