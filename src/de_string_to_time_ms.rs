use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

use crate::common::dt_str_to_utc_time_ms;

// Convert a string or number to i64
#[allow(unused)]
pub fn de_string_to_utc_time_ms<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => dt_str_to_utc_time_ms(&s).map_err(de::Error::custom)?,
        _ => return Err(de::Error::custom("Expecting String or Number")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize)]
    struct TimeRec {
        #[serde(rename = "Time")]
        #[serde(deserialize_with = "de_string_to_utc_time_ms")]
        time: i64,
    }

    #[test]
    fn test_de_string_to_utc_time_ms_json() {
        let js = r#"{ "Time": "2021-01-01 00:01:10" }"#;
        println!("{js}");
        let ap: TimeRec = serde_json::from_str(js).expect("Error de from str");
        println!("{:#?}", ap);

        assert_eq!(ap.time, 1609488070000);
    }

    #[test]
    fn test_de_string_to_utc_time_ms_csv() {
        let csv = "
Time
2021-01-01 00:01:10
2021-01-01 00:01:10.123";

        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let csv_result1: Option<Result<TimeRec, csv::Error>> = reader.deserialize().next();
        if let Some(result1) = csv_result1 {
            if let Ok(result1) = result1 {
                println!("{result1:?}");
                assert_eq!(result1.time, 1609488070000);
            }
        }

        let csv_result2: Option<Result<TimeRec, csv::Error>> = reader.deserialize().next();
        if let Some(result2) = csv_result2 {
            if let Ok(result2) = result2 {
                println!("{result2:?}");
                assert_eq!(result2.time, 1609488070123);
            }
        }
    }
}
