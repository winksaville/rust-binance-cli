use reqwest::header::HeaderMap;
use serde::{ser::SerializeMap, /*de, Deserialize,*/ Deserializer, /*Serialize,*/ Serializer,};

//use serde_json::Value;

//use crate::common::{dt_str_to_utc_time_ms, time_ms_to_utc_string, TzMassaging::CondAddTzUtc};

// Convert a string to UTC time in ms as i64
#[allow(unused)]
pub fn de_header_map<'de, D>(deserializer: D) -> Result<Option<HeaderMap>, D::Error>
where
    D: Deserializer<'de>,
{
    //println!("de_header_map");
    //Ok(None)
    panic!("Not yet implemented")

    // Could be something like this, which is from de_vec_balances_to_hashmap;

    //struct ItemsVisitor;

    //impl<'de> Visitor<'de> for ItemsVisitor {
    //    type Value = BTreeMap<String, Balance>;

    //    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    //        formatter.write_str("a sequence of items")
    //    }

    //    fn visit_seq<V>(self, mut seq: V) -> Result<BTreeMap<String, Balance>, V::Error>
    //    where
    //        V: SeqAccess<'de>,
    //    {
    //        let mut map: BTreeMap<String, Balance> = BTreeMap::new();
    //        //BTreeMap::with_capacity(seq.size_hint().unwrap_or(0));

    //        while let Some(item) = seq.next_element::<Balance>()? {
    //            // println!("item={:#?}", item);
    //            map.insert(item.asset.clone(), item);
    //        }

    //        Ok(map)
    //    }
    //}

    //deserializer.deserialize_seq(ItemsVisitor)
}

// Serialize HeaderMap
#[allow(unused)]
pub fn se_header_map<S>(headers: &Option<HeaderMap>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    println!("se_header_map:+");
    match headers {
        Some(h) => {
            println!("se_header_map: some");
            if !h.is_empty() {
                //println!("se_header_map: len > 0");
                let mut hm_serializer = s.serialize_map(None)?;
                let hm = h;
                for (k, v) in hm {
                    let val = match v.to_str() {
                        Ok(v) => v,
                        Err(e) => panic!("Could not convert header to string while serializing"), // Don't know how to convert to S::Error?
                    };
                    //if let Some(key) = k {
                    hm_serializer.serialize_entry(&k.to_string(), val)?;
                    //}
                }
                hm_serializer.end()
            } else {
                //println!("se_header_map: empty");
                let hm_serializer = s.serialize_map(None)?;
                let done: Result<<S as Serializer>::Ok, _> = hm_serializer.end();
                done
            }
        }
        None => {
            //println!("se_header_map: None");
            panic!("#[serde(skip_serializing_if = \"Option::is_none\")] maybe missing a HeaderMap field, probably in ResponseErrorRec");
        }
    }
}

//#[cfg(test)]
//mod test {
//    use super::*;
//    use serde::Serialize;
//
//    #[derive(Debug, Serialize, Deserialize)]
//    struct TimeRec {
//        #[serde(rename = "Time")]
//        #[serde(deserialize_with = "de_string_to_utc_time_ms_condaddtzutc")]
//        #[serde(serialize_with = "se_time_ms_to_utc_string")]
//        time: i64,
//    }
//
//    #[test]
//    fn test_de_string_to_utc_time_ms_json() {
//        let js = r#"{ "Time": "1970-01-01 00:00:00" }"#;
//        dbg!(js);
//        let ap: TimeRec = serde_json::from_str(js).expect("Error de from str");
//        dbg!(&ap);
//
//        assert_eq!(ap.time, 0);
//    }
//
//    #[test]
//    fn test_de_string_to_utc_time_ms_csv() {
//        let csv = "
//Time
//1970-01-01 00:00:00
//1970-01-01 00:00:00.123";
//
//        let mut reader = csv::Reader::from_reader(csv.as_bytes());
//        for (idx, entry) in reader.deserialize().enumerate() {
//            match entry {
//                Ok(tr) => {
//                    let tr: TimeRec = tr;
//                    println!("tr: {:?}", tr);
//                    match idx {
//                        0 => assert_eq!(tr.time, 0),
//                        1 => assert_eq!(tr.time, 123),
//                        _ => panic!("Unexpected idx"),
//                    }
//                }
//                Err(e) => panic!("Error: {e}"),
//            }
//        }
//    }
//
//    #[test]
//    fn test_se_time_ms_to_utc_string() {
//        let trs = vec![TimeRec { time: 0 }, TimeRec { time: 123 }];
//
//        let mut wtr = csv::Writer::from_writer(vec![]);
//        for tr in trs.iter() {
//            wtr.serialize(tr).expect("Error serializing");
//        }
//
//        let vec = wtr.into_inner().expect("Unexpected into Vec<u8>");
//        let data = String::from_utf8(vec).expect("Unexpected convert vec to String");
//        dbg!(&data);
//
//        assert_eq!(
//            data,
//            "Time\n1970-01-01T00:00:00.000+00:00\n1970-01-01T00:00:00.123+00:00\n"
//        );
//    }
//}
//
//
