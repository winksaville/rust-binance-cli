use serde::{de, Deserialize, Deserializer, /*Serialize,*/ Serializer};
use serde_json::Value;

use crate::common::{dt_str_to_utc_time_ms, time_ms_to_utc_string, TzMassaging::CondAddTzUtc};

// Convert a string to UTC time in ms as i64
#[allow(unused)]
pub fn de_string_to_utc_time_ms_condaddtzutc<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => dt_str_to_utc_time_ms(&s, CondAddTzUtc).map_err(de::Error::custom)?,
        _ => return Err(de::Error::custom("Expecting String or Number")),
    })
}

// Convert a string to UTC time in ms as i64
#[allow(unused)]
pub fn se_time_ms_to_utc_string<S>(time_ms: &i64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&time_ms_to_utc_string(*time_ms))
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize)]
    struct TimeRec {
        #[serde(rename = "Time")]
        #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
        #[serde(serialize_with = "se_time_ms_to_utc_string")]
        time: i64,
    }

    #[test]
    fn test_de_string_to_utc_time_ms_json() {
        let js = r#"{ "Time": "1970-01-01 00:00:00" }"#;
        dbg!(js);
        let ap: TimeRec = serde_json::from_str(js).expect("Error de from str");
        dbg!(&ap);

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

    #[test]
    fn test_se_time_ms_to_utc_string() {
        let trs = vec![TimeRec { time: 0 }, TimeRec { time: 123 }];

        let mut wtr = csv::Writer::from_writer(vec![]);
        for tr in trs.iter() {
            wtr.serialize(tr).expect("Error serializing");
        }

        let vec = wtr.into_inner().expect("Unexpected into Vec<u8>");
        let data = String::from_utf8(vec).expect("Unexpected convert vec to String");
        dbg!(&data);

        assert_eq!(
            data,
            "Time\n1970-01-01T00:00:00.000+00:00\n1970-01-01T00:00:00.123+00:00\n"
        );
    }
}
