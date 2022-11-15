
pub enum ColumnKind {
    /// Allows to build custom types for further interpretation
    Other(String),
    String,
    Number,
    AccountId,
    JSON,
    Bytes
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Tables,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Column {
    name: String,
    kind: ColumnKind,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Row {

}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Table {
    columns: Vec<Column>,
}
