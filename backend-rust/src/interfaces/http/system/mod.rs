use serde::Deserialize;

pub mod dept;
pub mod menu;
pub mod role;

#[derive(Debug, Deserialize)]
pub struct IdsReq {
    #[serde(default)]
    pub ids: Vec<i64>,
}
