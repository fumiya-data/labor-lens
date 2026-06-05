//! PostgreSQL execution adapter for the shared DB command model.
//!
//! `super` owns the stable command shape. This module owns the driver mapping,
//! transaction boundary, and JSON/typed bind conversion needed to execute those
//! commands against a live PostgreSQL database.

use super::{SqlCommand, SqlParam, SqlValue};
use ::postgres::types::ToSql;
use ::postgres::{Client, Config, NoTls};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

pub struct PostgresCommandAdapter {
    client: Client,
}

impl PostgresCommandAdapter {
    pub fn connect(connection_string: &str) -> Result<Self, PostgresAdapterError> {
        let config = Config::from_str(connection_string)?;
        let client = config.connect(NoTls)?;
        Ok(Self { client })
    }

    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub fn execute(&mut self, command: &SqlCommand) -> Result<u64, PostgresAdapterError> {
        let params = PostgresBindParams::try_from(command.params())?;
        Ok(self
            .client
            .execute(command.statement(), params.as_refs().as_slice())?)
    }

    pub fn execute_transaction(
        &mut self,
        commands: &[SqlCommand],
    ) -> Result<Vec<u64>, PostgresAdapterError> {
        let mut transaction = self.client.transaction()?;
        let mut affected_rows = Vec::with_capacity(commands.len());

        for command in commands {
            let params = PostgresBindParams::try_from(command.params())?;
            affected_rows
                .push(transaction.execute(command.statement(), params.as_refs().as_slice())?);
        }

        transaction.commit()?;
        Ok(affected_rows)
    }

    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

#[derive(Debug)]
struct PostgresBindParams {
    values: Vec<PostgresBindValue>,
}

impl PostgresBindParams {
    fn try_from(params: &[SqlParam]) -> Result<Self, PostgresAdapterError> {
        let values = params
            .iter()
            .map(PostgresBindValue::try_from_param)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { values })
    }

    fn as_refs(&self) -> Vec<&(dyn ToSql + Sync)> {
        self.values
            .iter()
            .map(|value| value.as_to_sql())
            .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
enum PostgresBindValue {
    Null(Option<String>),
    Text(String),
    Int64(i64),
    Int32(i32),
    Bool(bool),
    Json(serde_json::Value),
}

impl PostgresBindValue {
    fn try_from_param(param: &SqlParam) -> Result<Self, PostgresAdapterError> {
        match param.sql_value() {
            SqlValue::Null => Ok(Self::Null(None)),
            SqlValue::Text(value) => Ok(Self::Text(value.clone())),
            SqlValue::Int64(value) => Ok(Self::Int64(*value)),
            SqlValue::Int32(value) => Ok(Self::Int32(*value)),
            SqlValue::Bool(value) => Ok(Self::Bool(*value)),
            SqlValue::Json(value) => {
                serde_json::from_str(value)
                    .map(Self::Json)
                    .map_err(|source| PostgresAdapterError::InvalidJsonParam {
                        name: param.name(),
                        source,
                    })
            }
        }
    }

    fn as_to_sql(&self) -> &(dyn ToSql + Sync) {
        match self {
            Self::Null(value) => value,
            Self::Text(value) => value,
            Self::Int64(value) => value,
            Self::Int32(value) => value,
            Self::Bool(value) => value,
            Self::Json(value) => value,
        }
    }
}

#[derive(Debug)]
pub enum PostgresAdapterError {
    Driver(::postgres::Error),
    InvalidJsonParam {
        name: &'static str,
        source: serde_json::Error,
    },
}

impl fmt::Display for PostgresAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Driver(source) => write!(formatter, "PostgreSQL command failed: {source}"),
            Self::InvalidJsonParam { name, source } => {
                write!(formatter, "invalid JSON bind parameter `{name}`: {source}")
            }
        }
    }
}

impl Error for PostgresAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Driver(source) => Some(source),
            Self::InvalidJsonParam { source, .. } => Some(source),
        }
    }
}

impl From<::postgres::Error> for PostgresAdapterError {
    fn from(source: ::postgres::Error) -> Self {
        Self::Driver(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::db::{InsertInputRef, InsertRunRecord};

    #[test]
    fn bind_params_preserve_schema_types_for_input_refs() {
        let command = InsertInputRef::new(
            "src_01J00000000000000000000000",
            "run_01J00000000000000000000000",
            "attendance",
            "attendance.csv",
            "source://attendance.csv",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            120,
            "utf-8",
            ",",
            true,
            "attendance_csv.v1",
        )
        .with_detected_shape(2, 5)
        .to_sql_command();

        assert!(matches!(
            command.params()[6].sql_value(),
            SqlValue::Int64(120)
        ));
        assert!(matches!(
            command.params()[9].sql_value(),
            SqlValue::Bool(true)
        ));
        assert!(matches!(
            command.params()[11].sql_value(),
            SqlValue::Int64(2)
        ));
        assert!(matches!(
            command.params()[12].sql_value(),
            SqlValue::Int32(5)
        ));
    }

    #[test]
    fn json_params_are_validated_before_driver_execution() {
        let command = InsertRunRecord::new(
            "run_01J00000000000000000000000",
            "tenant_01J000000000000000000000",
            "created",
            "unknown",
            "local_db.v1",
        )
        .with_settings_json("{not valid json")
        .to_sql_command();

        let error = PostgresBindParams::try_from(command.params()).unwrap_err();

        assert!(matches!(
            error,
            PostgresAdapterError::InvalidJsonParam {
                name: "settings",
                ..
            }
        ));
    }
}
