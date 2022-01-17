use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

use crate::common::{dt_str_to_utc_time_ms, TzMassaging::AddTzUtc};

// Convert a string to UTC time in ms as i64
#[allow(unused)]
pub fn de_string_to_utc_time_ms<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => dt_str_to_utc_time_ms(&s, AddTzUtc).map_err(de::Error::custom)?,
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
        let js = r#"{ "Time": "1970-01-01 00:00:00" }"#;
        println!("{js}");
        let ap: TimeRec = serde_json::from_str(js).expect("Error de from str");
        println!("{:#?}", ap);

        assert_eq!(ap.time, 0);
    }

    #[test]
    fn test_de_string_to_utc_time_ms_csv() {
        let csv = "
Time
1970-01-01 00:00:00
1970-01-01 00:00:00.123";

        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for (idx, entry) in reader.deserialize().enumerate() {
            match entry {
                Ok(tr) => {
                    let tr: TimeRec = tr;
                    println!("tr: {:?}", tr);
                    match idx {
                        0 => assert_eq!(tr.time, 0),
                        1 => assert_eq!(tr.time, 123),
                        _ => panic!("Unexpected idx"),
                    }
                }
                Err(e) => panic!("Error: {e}"),
            }
        }
    }
}
