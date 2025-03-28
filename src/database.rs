use sqlx::{Column, PgPool, Row, TypeInfo};
use sqlx_postgres::PgPoolOptions;

use sqlx::postgres::types::{PgInterval, PgMoney};

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

use crate::data::structs::ValueType;

use indexmap::IndexMap;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(url: impl ToString) -> Result<Self, String> {
        let pool = PgPoolOptions::new().connect(&url.to_string()).await;

        if let Ok(pool) = pool {
            return Ok(Self { pool });
        } else if let Err(e) = pool {
            return Err(e.to_string());
        }

        Err(String::from("Unknown error"))
    }

    pub async fn get_tables(&self) -> Result<Vec<String>, String> {
        let rows = sqlx::query("SELECT tablename FROM pg_tables WHERE schemaname NOT IN ('pg_catalog', 'information_schema') AND schemaname NOT LIKE 'pg_%'")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let tables = rows
            .into_iter()
            .map(|row| row.try_get("tablename").map_err(|e| e.to_string()))
            .collect::<Result<Vec<String>, String>>()?;

        Ok(tables)
    }

    pub async fn get_databases(&self) -> Result<Vec<String>, String> {
        let rows = sqlx::query("SELECT datname FROM pg_database")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let databases = rows
            .into_iter()
            .map(|row| row.try_get("datname").map_err(|e| e.to_string()))
            .collect::<Result<Vec<String>, String>>()?;

        Ok(databases)
    }

    pub async fn execute_query(
        &self,
        query: &str,
    ) -> Result<IndexMap<String, Vec<ValueType>>, String> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut results = indexmap::IndexMap::new();

        for row in rows {
            for column in row.columns() {
                let column_name = column.name().to_string();
                let type_info = column.type_info();
                let type_name = type_info.name();

                let value = match type_name {
                    // Numeric types
                    "INT2" | "SMALLINT" => row
                        .try_get::<i16, _>(column_name.as_str())
                        .map(|v| ValueType::Int(v as i32))
                        .unwrap_or(ValueType::Null),
                    "INT4" | "INTEGER" => row
                        .try_get::<i32, _>(column_name.as_str())
                        .map(ValueType::Int)
                        .unwrap_or(ValueType::Null),
                    "INT8" | "BIGINT" => row
                        .try_get::<i64, _>(column_name.as_str())
                        .map(ValueType::BigInt)
                        .unwrap_or(ValueType::Null),
                    "FLOAT4" | "REAL" => row
                        .try_get::<f32, _>(column_name.as_str())
                        .map(|v| ValueType::Float(v as f64))
                        .unwrap_or(ValueType::Null),
                    "FLOAT8" | "DOUBLE PRECISION" => row
                        .try_get::<f64, _>(column_name.as_str())
                        .map(ValueType::Float)
                        .unwrap_or(ValueType::Null),
                    "MONEY" => row
                        .try_get::<PgMoney, _>(column_name.as_str())
                        .map(|v| ValueType::BigInt(v.0))
                        .unwrap_or(ValueType::Null),

                    // Text types
                    "CHAR" | "VARCHAR" | "TEXT" | "NAME" => row
                        .try_get::<String, _>(column_name.as_str())
                        .map(ValueType::Text)
                        .unwrap_or(ValueType::Null),
                    "BPCHAR" => row
                        .try_get::<String, _>(column_name.as_str())
                        .map(|s| ValueType::Text(s.trim_end().to_string()))
                        .unwrap_or(ValueType::Null),

                    // Boolean
                    "BOOL" | "BOOLEAN" => row
                        .try_get::<bool, _>(column_name.as_str())
                        .map(ValueType::Bool)
                        .unwrap_or(ValueType::Null),

                    // Binary
                    "BYTEA" => row
                        .try_get::<Vec<u8>, _>(column_name.as_str())
                        .map(|bytes| {
                            String::from_utf8(bytes)
                                .map(ValueType::Text)
                                .unwrap_or_else(|e| ValueType::Bytea(e.into_bytes()))
                        })
                        .unwrap_or(ValueType::Null),

                    // Date/Time types
                    "TIMESTAMP" | "TIMESTAMPTZ" => row
                        .try_get::<DateTime<Utc>, _>(column_name.as_str())
                        .map(|v| ValueType::Text(v.to_rfc3339()))
                        .unwrap_or(ValueType::Null),
                    "DATE" => row
                        .try_get::<NaiveDate, _>(column_name.as_str())
                        .map(|v| ValueType::Text(v.to_string()))
                        .unwrap_or(ValueType::Null),
                    "TIME" | "TIMETZ" => row
                        .try_get::<NaiveTime, _>(column_name.as_str())
                        .map(|v| ValueType::Text(v.to_string()))
                        .unwrap_or(ValueType::Null),
                    "INTERVAL" => row
                        .try_get::<PgInterval, _>(column_name.as_str())
                        .map(|_| ValueType::Text("interval".to_string())) // Simplified representation
                        .unwrap_or(ValueType::Null),

                    // JSON types
                    "JSON" | "JSONB" => row
                        .try_get::<serde_json::Value, _>(column_name.as_str())
                        .map(|v| ValueType::Text(v.to_string()))
                        .unwrap_or(ValueType::Null),

                    // Array types (basic handling)
                    typ if typ.starts_with("_") => row
                        .try_get::<Vec<String>, _>(column_name.as_str())
                        .map(|v| ValueType::Array(v.into_iter().map(ValueType::Text).collect()))
                        .unwrap_or(ValueType::Null),

                    // Unknown/special types that need special handling
                    "ACLITEM" => ValueType::Text("[ACL permissions]".to_string()),

                    // Unknown types
                    _ => ValueType::Unknown(type_name.to_string()),
                };

                if results.get(&column_name).is_none() {
                    results.insert(column_name.clone(), Vec::new());
                }
                results.get_mut(&column_name).unwrap().push(value);
            }
        }

        Ok(results)
    }
}
