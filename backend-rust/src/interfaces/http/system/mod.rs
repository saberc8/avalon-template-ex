use serde::{de, Deserialize, Deserializer};

pub mod client;
pub mod dept;
pub mod dict;
pub mod file;
pub mod menu;
pub mod option;
pub mod role;
pub mod storage;
pub mod user;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdList(pub Vec<i64>);

#[derive(Debug, Deserialize)]
pub struct IdsReq {
    #[serde(default, deserialize_with = "deserialize_id_vec")]
    pub ids: Vec<i64>,
}

impl<'de> Deserialize<'de> for IdList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_id_vec(deserializer).map(Self)
    }
}

fn deserialize_id_vec<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<IdValue>::deserialize(deserializer)?;
    values
        .into_iter()
        .map(IdValue::into_i64)
        .collect::<Result<Vec<_>, D::Error>>()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IdValue {
    Number(i64),
    String(String),
}

impl IdValue {
    fn into_i64<E>(self) -> Result<i64, E>
    where
        E: de::Error,
    {
        match self {
            Self::Number(value) => Ok(value),
            Self::String(value) => value
                .trim()
                .parse::<i64>()
                .map_err(|_| E::custom("ID must be an integer string or number")),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn id_list_accepts_string_and_number_ids() {
        let ids: IdList = serde_json::from_value(json!(["123", 456, " 789 "])).unwrap();

        assert_eq!(ids.0, vec![123, 456, 789]);
    }

    #[test]
    fn ids_request_accepts_string_and_number_ids() {
        let req: IdsReq = serde_json::from_value(json!({"ids": ["123", 456]})).unwrap();

        assert_eq!(req.ids, vec![123, 456]);
    }
}
