use serde::{Deserialize, Deserializer};
use time::{Date, format_description::well_known::Iso8601};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Date::parse(&s, &Iso8601::DATE).map_err(serde::de::Error::custom)
}
