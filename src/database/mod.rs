use chashmap::CHashMap;
use std::fs::File;

mod table;
mod table_schema;
mod table_value;

#[cfg(test)]
extern crate mockers_derive;

#[cfg(test)]
use mockers_derive::mocked;

pub use table_schema::Schema;
pub use table_value::TableValue;

pub struct RecordID {
    pub page_number: u32,
    pub slot_id: u8,
}

impl RecordID {
    pub fn new(page_number: u32, slot_id: u8) -> RecordID {
        RecordID {
            page_number,
            slot_id,
        }
    }
}

#[cfg_attr(test, mocked)]
pub trait Table {
    fn insert(&self, row: Vec<TableValue>) -> Result<RecordID, String>;
}

pub trait Factory<T: Table> {
    fn new_table(&self, schema: Schema) -> Result<T, String>;
}

pub struct Database<T: Table, F: Factory<T>> {
    directory: File,
    factory: F,
    tables: CHashMap<String, T>,
}

impl<T: Table, F: Factory<T>> Database<T, F> {
    /// Creates a new database
    pub fn new(directory: File, factory: F) -> Database<T, F> {
        Database {
            directory,
            factory,
            tables: CHashMap::new(),
        }
    }

    /// Creates a new table in the database
    pub fn create_table(&self, schema: Schema) -> Result<(), String> {
        let table_name = schema.table_name.clone();
        if self.tables.contains_key(&table_name) {
            return Err(format!("table {} already exists", &table_name).to_string());
        }

        let new_table = self.factory.new_table(schema)?;
        self.tables.insert_new(table_name, new_table);

        Ok(())
    }

    pub fn insert(&self, table_name: &str, row: Vec<TableValue>) -> Result<(), String> {
        if self.tables.get(table_name).is_none() {
            return Err(format!("no such table: {}", table_name));
        }

        let table = self.tables.get(table_name).unwrap();
        let record_id = table.insert(row)?;

        Ok(())
    }
}

#[cfg(test)]
mod mocks {
    use super::*;
    pub struct MockFactory<T: Table, F: Fn() -> T> {
        pub next_table: F,
    }

    impl<T: Table, F: Fn() -> T> MockFactory<T, F> {
        pub fn new(next_table: F) -> MockFactory<T, F> {
            MockFactory { next_table }
        }
    }

    impl<T: Table, F: Fn() -> T> Factory<T> for MockFactory<T, F> {
        fn new_table(&self, _: Schema) -> Result<T, String> {
            let next_table = &self.next_table;
            Ok(next_table())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockers::Scenario;

    #[test]
    fn test_unique_table_constraint() {
        let scenario = Scenario::new();

        let database = Database::new(
            tempfile::tempfile().unwrap(),
            mocks::MockFactory::new(|| {
                let (table, _) = scenario.create_mock_for::<dyn Table>();
                table
            }),
        );

        let table_name = "bank_accounts";
        database
            .create_table(Schema::new(table_name, vec![]))
            .expect("first table should be inserted");
        database
            .create_table(Schema::new(table_name, vec![]))
            .expect_err("table with the same name should not be inserted");
    }
}
