use std::io::Write;

use diesel::{AsExpression, deserialize, FromSqlRow, QueryId, serialize, SqlType};
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};
use serde::{Deserialize, Serialize};

/// Represent the list of fields that can be used to filter agents
#[derive(Debug, Clone, PartialEq, FromSqlRow, QueryId, AsExpression, SqlType, Eq)]
#[diesel(postgres_type(name = "agent_fields"), sql_type = AgentFields)]
pub enum AgentFields {
	OperativeSystem,
	Hostname,
	Domain,
	Username,
	Ip,
	ProcessId,
	ParentProcessId,
	ProcessName,
	ServerSecretKey,
	SecretKey,
	Signature,
	IntegrityLevel,
	Cwd,
}

impl ToSql<AgentFields, Pg> for AgentFields {
	fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
		match *self {
			Self::OperativeSystem => out.write_all(b"operative_system")?,
			Self::Hostname => out.write_all(b"hostname")?,
			Self::Domain => out.write_all(b"domain")?,
			Self::Username => out.write_all(b"username")?,
			Self::Ip => out.write_all(b"ip")?,
			Self::ProcessId => out.write_all(b"process_id")?,
			Self::ParentProcessId => out.write_all(b"parent_process_id")?,
			Self::ProcessName => out.write_all(b"process_name")?,
			Self::ServerSecretKey => out.write_all(b"server_secret_key")?,
			Self::SecretKey => out.write_all(b"secret_key")?,
			Self::Signature => out.write_all(b"signature")?,
			Self::IntegrityLevel => out.write_all(b"integrity_level")?,
			Self::Cwd => out.write_all(b"cwd")?,
		}
		Ok(IsNull::No)
	}
}

impl FromSql<AgentFields, Pg> for AgentFields {
	fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
		match bytes.as_bytes() {
			b"operative_system" => Ok(Self::OperativeSystem),
			b"hostname" => Ok(Self::Hostname),
			b"domain" => Ok(Self::Domain),
			b"username" => Ok(Self::Username),
			b"ip" => Ok(Self::Ip),
			b"process_id" => Ok(Self::ProcessId),
			b"parent_process_id" => Ok(Self::ParentProcessId),
			b"process_name" => Ok(Self::ProcessName),
			b"integrity_level" => Ok(Self::IntegrityLevel),
			b"cwd" => Ok(Self::Cwd),
			b"server_secret_key" => Ok(Self::ServerSecretKey),
			b"secret_key" => Ok(Self::SecretKey),
			b"signature" => Ok(Self::Signature),
			_ => Err("Unrecognized enum variant".into()),
		}
	}
}

/// Represent the list of valid logical operators that can be used for filtering
#[derive(Debug, Clone, PartialEq, FromSqlRow, QueryId, SqlType, AsExpression, Eq)]
#[diesel(postgres_type(name = "logical_operator"), sql_type = LogicalOperator)]
pub enum LogicalOperator {
	And,
	Or,
}

impl ToSql<LogicalOperator, Pg> for LogicalOperator {
	fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
		match *self {
			Self::And => out.write_all(b"and")?,
			Self::Or => out.write_all(b"or")?,
		}
		Ok(IsNull::No)
	}
}

impl FromSql<LogicalOperator, Pg> for LogicalOperator {
	fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
		match bytes.as_bytes() {
			b"and" => Ok(Self::And),
			b"or" => Ok(Self::Or),
			_ => Err("Unrecognized enum variant".into()),
		}
	}
}

/// Represent the list of valid operators that can be used for filtering
#[derive(Debug, Clone, PartialEq, FromSqlRow, QueryId, SqlType, AsExpression, Eq)]
#[diesel(postgres_type(name = "filter_operator"), sql_type = FilterOperator)]
pub enum FilterOperator {
	Equals,
	NotEquals,
	Contains,
	NotContains,
	StartsWith,
	EndsWith,
}

impl ToSql<FilterOperator, Pg> for FilterOperator {
	fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
		match *self {
			Self::Equals => out.write_all(b"equals")?,
			Self::NotEquals => out.write_all(b"not_equals")?,
			Self::Contains => out.write_all(b"contains")?,
			Self::NotContains => out.write_all(b"not_contains")?,
			Self::StartsWith => out.write_all(b"starts_with")?,
			Self::EndsWith => out.write_all(b"ends_with")?,
		}
		Ok(IsNull::No)
	}
}

impl FromSql<FilterOperator, Pg> for FilterOperator {
	fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
		match bytes.as_bytes() {
			b"equals" => Ok(Self::Equals),
			b"not_equals" => Ok(Self::NotEquals),
			b"contains" => Ok(Self::Contains),
			b"not_contains" => Ok(Self::NotContains),
			b"starts_with" => Ok(Self::StartsWith),
			b"ends_with" => Ok(Self::EndsWith),
			_ => Err("Unrecognized enum variant".into()),
		}
	}
}

/// Represent the list of valid log levels
#[derive(
	Debug,
	Clone,
	PartialEq,
	FromSqlRow,
	QueryId,
	SqlType,
	AsExpression,
	Eq,
	Serialize,
	Deserialize
)]
#[diesel(postgres_type(name = "log_level"), sql_type = LogLevel)]
pub enum LogLevel {
	#[serde(rename = "INFO")]
	INFO,
	#[serde(rename = "WARN")]
	WARN,
	#[serde(rename = "ERROR")]
	ERROR,
	#[serde(rename = "DEBUG")]
	DEBUG,
	#[serde(rename = "TRACE")]
	TRACE,
}

impl ToSql<LogLevel, Pg> for LogLevel {
	fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
		match *self {
			Self::INFO => out.write_all(b"INFO")?,
			Self::WARN => out.write_all(b"WARN")?,
			Self::ERROR => out.write_all(b"ERROR")?,
			Self::DEBUG => out.write_all(b"DEBUG")?,
			Self::TRACE => out.write_all(b"TRACE")?,
		}
		Ok(IsNull::No)
	}
}

impl FromSql<LogLevel, Pg> for LogLevel {
	fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
		match bytes.as_bytes() {
			b"INFO" => Ok(Self::INFO),
			b"WARN" => Ok(Self::WARN),
			b"ERROR" => Ok(Self::ERROR),
			b"DEBUG" => Ok(Self::DEBUG),
			b"TRACE" => Ok(Self::TRACE),
			_ => Err("Unrecognized enum variant".into()),
		}
	}
}
