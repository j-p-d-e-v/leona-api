#[derive(Debug, Clone)]
pub enum Error {
    DbConnErr(String),
    DbConnOptErr(String),
    DbQueryErr(String),
}
