use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MetaInfoFile {
    pub announce: String,
    pub info: InfoSingleFile,
    pub created_by: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InfoSingleFile {
    pub name: String,
    pub length: i64,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    pub pieces: String,
}
