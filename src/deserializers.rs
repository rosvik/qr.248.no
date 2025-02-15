use serde::{de, Deserialize, Deserializer};
use std::{fmt, str::FromStr};

pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

pub fn string_as_bool<'de, D>(de: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = String::deserialize(de).unwrap_or("off".to_string());
    match opt.as_str() {
        "on" | "true" | "" => Ok(true),
        "off" | "false" => Ok(false),
        _ => Ok(false),
    }
}
