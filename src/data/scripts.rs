pub const CREATE_TABLE: &str = r#"CREATE TABLE ...
(
    ...
)"#;

pub const CREATE_INDEX: &str = r#"CREATE INDEX ... ON ...
(
    ...
)"#;

pub const DROP_TABLE: &str = r#"DROP TABLE {table_name}"#;

pub const INSERT: &str = r#"INSERT INTO {table_name}
(...)
VALUES (...)"#;

pub const UPDATE: &str = r#"UPDATE {table_name}
SET ... = ...
WHERE ..."#;

pub const DELETE: &str = r#"DELETE FROM {table_name}
WHERE ..."#;

pub const SELECT: &str = r#"SELECT *
FROM {table_name}"#;

pub const SELECT_100: &str = r#"SELECT *
FROM {table_name}
LIMIT 100"#;

pub const GET_TABLE_COLUMNS: &str = r#"SELECT column_name
FROM information_schema.columns
WHERE table_name = '{table_name}'"#;
