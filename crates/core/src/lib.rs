mod error;
mod row;
mod table;
use std::collections::HashMap;

use derive_more::Display;
pub use error::{QueryExecutionError, SQLError};
use parser::ast::{parse_multiple_queries, parse_sql_query, SqlQuery};
use table::{Table, TableIter};

#[derive(Debug, Display)]

pub enum ExecResponse<'a> {
    #[display(fmt = "{_0:#?}")] // only show the values not "Select(...)"
    Select(TableIter<'a>),
    Insert,
    Create,
}

#[derive(Debug, Default)]
pub struct Execution {
    tables: HashMap<String, Table>,
}

impl Execution {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn run(&mut self, query: SqlQuery) -> Result<ExecResponse, QueryExecutionError> {
        match query {
            SqlQuery::Select(select) => {
                let columns = select.fields;
                let table = select.table;
                let table = self
                    .tables
                    .get(&table)
                    .ok_or(QueryExecutionError::TableNotFound(table))?;
                Ok(ExecResponse::Select(table.select(columns)?))
            }
            SqlQuery::Insert(insert) => {
                let Some(table) = self.tables.get_mut(&insert.table) else {
                    return Err(QueryExecutionError::TableNotFound(insert.table))
                };

                table.insert(insert.values)?;
                Ok(ExecResponse::Insert)
            }
            SqlQuery::Create(create) => {
                let table = Table::new(create.columns);
                if self.tables.contains_key(&create.table) {
                    return Err(QueryExecutionError::TableAlreadyExists(create.table));
                }
                self.tables.insert(create.table, table);
                Ok(ExecResponse::Create)
            }
        }
    }

    pub fn parse_and_run<'a>(&mut self, query: &'a str) -> Result<ExecResponse, SQLError<'a>> {
        let query = parse_sql_query(query)?;

        let res = self.run(query)?;
        Ok(res)
    }

    pub fn parse_multiple_and_run<'a>(
        &'a mut self,
        query: &'a str,
    ) -> Result<ExecResponse, SQLError<'a>> {
        let queries = parse_multiple_queries(query)?;

        let (last, rest) = queries
            .split_last()
            .expect("at least one query should have been parsed");

        for q in rest {
            self.run(q.clone())?;
        }

        let res = self.run(last.clone())?;
        Ok(res)
    }
}
