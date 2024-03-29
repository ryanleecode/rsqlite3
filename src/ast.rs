use crate::{executor, table};
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Ast {
    Exit,
    Create(TableSchema),
    Insert(Insertion),
    Select(Selection),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ColumnSet {
    WildCard,
    Names(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum Value {
    Integer(i64),
    Null,
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Integer(_), Value::Null) => Ordering::Less,
            (Value::Null, Value::Integer(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Selection {
    table_name: String,
    columns: ColumnSet,
}

impl executor::Selection for Selection {
    fn table_name(&self) -> &String {
        &self.table_name
    }

    fn validate(&self) -> Result<(), String> {
        self.validate()
    }

    fn columns(&self) -> ColumnSet {
        self.columns()
    }
}

impl Selection {
    pub fn new(table_name: &str, columns: ColumnSet) -> Selection {
        Selection {
            table_name: table_name.to_string(),
            columns,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        return Ok(());
    }

    fn columns(&self) -> ColumnSet {
        self.columns.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub is_primary_key: bool,
}

impl Column {
    pub fn new(name: &str, is_primary_key: bool) -> Column {
        Column {
            name: name.to_string(),
            is_primary_key,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
}

impl TableSchema {
    pub fn new(name: &str, columns: Vec<Column>) -> TableSchema {
        TableSchema {
            name: name.to_string(),
            columns,
        }
    }
}

impl table::TableSchema for TableSchema {
    fn table_name(&self) -> String {
        self.name.clone()
    }

    fn columns(&self) -> Vec<Column> {
        return self.columns.clone();
    }

    fn validate(&self) -> Result<(), String> {
        let mut column_names = HashSet::new();
        let mut has_primary_key = false;
        for c in &self.columns {
            if column_names.contains(&c.name) {
                return Err(format!("duplicate column name: {}", c.name));
            }
            if c.is_primary_key && has_primary_key {
                return Err(format!(
                    "table \"{}\" has more than one primary key",
                    self.name
                ));
            }
            if c.is_primary_key {
                has_primary_key = true
            }
            column_names.insert(c.name.clone());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Insertion {
    pub table_name: String,
    column_names: Option<Vec<String>>,
    values: Vec<Value>,
}

impl executor::Insertion for Insertion {
    fn table_name(&self) -> &String {
        &self.table_name
    }

    fn validate(&self) -> Result<(), String> {
        self.validate()
    }

    fn column_names(&self) -> Option<Box<dyn Iterator<Item = String>>> {
        self.column_names()
    }

    fn values(&self) -> Box<dyn Iterator<Item = Value>> {
        self.values()
    }
}

impl Insertion {
    pub fn new(
        table_name: &str,
        column_names: Option<Vec<String>>,
        values: Vec<Value>,
    ) -> Insertion {
        return Insertion {
            table_name: table_name.to_string(),
            column_names: column_names.map(|column_names| {
                column_names
                    .iter()
                    .map(|column_name| column_name.clone())
                    .collect()
            }),
            values,
        };
    }
    pub fn validate(&self) -> Result<(), String> {
        return self
            .column_names
            .as_ref()
            .map(|column_names| {
                if self.values.len() != column_names.len() {
                    return Err(format!(
                        "{} values for {} columns",
                        self.values.len(),
                        column_names.len()
                    ));
                }

                Ok(())
            })
            .map_or_else(|| Ok(()), |r| r);
    }

    pub fn column_names(&self) -> Option<Box<dyn Iterator<Item = String>>> {
        self.column_names.as_ref().and_then(|column_names| {
            Some(Box::new(column_names.clone().into_iter()) as Box<dyn Iterator<Item = String>>)
        })
    }

    pub fn values(&self) -> Box<dyn Iterator<Item = Value>> {
        Box::new(self.values.clone().into_iter())
    }
}

#[cfg(test)]
mod test_parsing {
    use super::*;
    use crate::sqlite3;

    #[test]
    fn insertion_statement() {
        let statement = "INSERT INTO apples(slices) VALUES(15);";
        let parse_result = sqlite3::AstParser::new().parse(statement);
        if parse_result.is_err() {
            parse_result.expect("should parse insertion statement");
        } else {
            let insert_stmt = parse_result.unwrap();
            assert_eq!(
                insert_stmt,
                Ast::Insert(Insertion::new(
                    "apples",
                    Some(vec!["slices".to_string()]),
                    vec![Value::Integer(15)],
                ))
            )
        }
    }

    #[test]
    fn create_table_statement() {
        let statement = "CREATE TABLE apples(slices INTEGER);";
        let parse_result = sqlite3::AstParser::new().parse(statement);
        if parse_result.is_err() {
            parse_result.expect("should parse create table statement");
        } else {
            let insert_stmt = parse_result.unwrap();
            assert_eq!(
                insert_stmt,
                Ast::Create(TableSchema {
                    name: "apples".to_string(),
                    columns: vec![Column::new("slices", false)]
                })
            )
        }
    }
}

#[cfg(test)]
mod test_insertion {
    use super::*;

    #[test]
    fn validation_fails_if_num_values_neq_num_columns() {
        let table_name = "eggs";
        let insertion = Insertion::new(
            table_name,
            Some(vec!["count".to_string()]),
            vec![Value::Integer(32), Value::Integer(1337)],
        );
        let result = insertion.validate();
        assert_eq!(result.is_err(), true);
    }
}

#[cfg(test)]
mod test_table_schema {
    use super::*;
    use crate::table::TableSchema;

    #[test]
    fn validation_fails_if_there_are_duplicate_column_names() {
        let table_schema = super::TableSchema::new(
            "kings",
            vec![Column::new("henry", false), Column::new("henry", false)],
        );

        let result = table_schema.validate();
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn validation_fails_if_there_are_duplicate_primary_keys() {
        let table_schema = super::TableSchema::new(
            "kings",
            vec![Column::new("henry", true), Column::new("james", true)],
        );

        let result = table_schema.validate();
        assert_eq!(result.is_err(), true);
    }
}
