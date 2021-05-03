use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

// TODO: Could these be combined and generalized into a single
//       generic implemenation over all iX, uX and fX numeric types?
//
// Convert a string or number to i64
pub fn de_string_or_number_to_i64<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<i64>().map_err(de::Error::custom)?,
        Value::Number(num) => num
            .as_i64()
            .ok_or_else(|| de::Error::custom("Invalid number as_i64"))?,
        _ => return Err(de::Error::custom("Expecting String or Number")),
    })
}

pub fn de_string_or_number_to_u64<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<u64>().map_err(de::Error::custom)?,
        Value::Number(num) => num
            .as_u64()
            .ok_or_else(|| de::Error::custom("Invalid number as_i64"))?,
        _ => return Err(de::Error::custom("Expecting String or Number")),
    })
}

// Convert a string or number to f64
#[allow(unused)]
pub fn de_string_or_number_to_f64<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<f64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<f64>().map_err(de::Error::custom)?,
        Value::Number(num) => num
            .as_f64()
            .ok_or_else(|| de::Error::custom("Invalid number as_i64"))?,
        _ => return Err(de::Error::custom("Expecting String or Number")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test::Bencher;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize)]
    struct ValuesToTest {
        #[serde(deserialize_with = "de_string_or_number_to_i64")]
        value_i64: i64,
        #[serde(deserialize_with = "de_string_or_number_to_u64")]
        value_u64: u64,
        #[serde(deserialize_with = "de_string_or_number_to_f64")]
        value_f64: f64,
    }

    #[test]
    fn test_de_string_or_number_from_numbers() {
        let js = r#"{ "value_i64": -1, "value_u64": 5, "value_f64": 1.2 }"#;
        let ap: ValuesToTest = serde_json::from_str(js).expect("Error de from str");
        assert_eq!(ap.value_i64, -1i64);
        assert_eq!(ap.value_u64, 5u64);
        assert_eq!(ap.value_f64, 1.2f64)
    }

    #[test]
    fn test_de_string_or_number_from_strings() {
        let js = r#"{ "value_i64": "-1", "value_u64": "5", "value_f64": "1.2" }"#;
        let ap: ValuesToTest = serde_json::from_str(js).expect("Error de from str");
        assert_eq!(ap.value_i64, -1i64);
        assert_eq!(ap.value_u64, 5u64);
        assert_eq!(ap.value_f64, 1.2f64)
    }

    #[test]
    fn test_de_sting_or_number_to_i64_errors() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Vi64ToTest {
            #[serde(deserialize_with = "de_string_or_number_to_i64")]
            value_i64: i64,
        }
        //let js_val = json!({"value_i64": null });
        //let js = js_val.to_string();
        //let js = r#"{ "value_i64": "a string" }"#;
        let js = r#"{ "value_i64": null }"#;
        println!("js={}", js);
        let ap: Result<Vi64ToTest, _> = serde_json::from_str(&js);
        println!("ap={:#?}", ap);
        match ap {
            Ok(_) => panic!("Should never happen"),
            Err(e) => {
                println!("ap Err e={:#?}", e);
                //assert!(e.to_string().contains("invalid type: map, expected i64"));
            }
        }
    }

    #[test]
    fn test_de_sting_or_number_to_u64_errors() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Vu64ToTest {
            #[serde(deserialize_with = "de_string_or_number_to_u64")]
            value_u64: u64,
        }
        let js = r#"{ "value_u64": null }"#;
        let ap: Result<Vu64ToTest, serde_json::Error> = serde_json::from_str(js);
        println!("ap={:#?}", ap);
        match ap {
            Ok(_) => panic!("Should never happen"),
            Err(e) => {
                assert!(e.to_string().contains("Expecting String or Number"));
            }
        }
    }

    #[test]
    fn test_de_sting_or_number_to_f64_errors() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Vf64ToTest {
            #[serde(deserialize_with = "de_string_or_number_to_f64")]
            value_f64: f64,
        }
        let js = r#"{ "value_f64": null }"#;
        let ap: Result<Vf64ToTest, serde_json::Error> = serde_json::from_str(js);
        println!("ap={:#?}", ap);
        match ap {
            Ok(_) => panic!("Should never happen"),
            Err(e) => {
                assert!(e.to_string().contains("Expecting String or Number"));
            }
        }
    }

    // Enable once feature(test) is in stable
    // #[bench]
    // fn bench_de_string_from_str_to_struct(b: &mut Bencher) {
    //     let js = r#"{ "value_i64": "-1", "value_u64": "5", "value_f64": "1.2" }"#;
    //     b.iter(|| {
    //         let ap: ValuesToTest = serde_json::from_str(js).expect("Error de from str");
    //         test::black_box(ap);
    //     });
    // }

    // #[bench]
    // fn bench_de_string_from_value_to_struct(b: &mut Bencher) {
    //     let js = r#"{ "value_i64": "-1", "value_u64": "5", "value_f64": "1.2" }"#;
    //     b.iter(|| {
    //         let jv = serde_json::from_str(js).expect("Error de from str");
    //         let ap: ValuesToTest = serde_json::from_value(jv).expect("Error de from str");
    //         test::black_box(ap);
    //     });
    // }

    // #[bench]
    // /// TODO: Why is this slower than `bench_de_string_from_str_to_struct`
    // fn bench_de_number_from_str_to_struct(b: &mut Bencher) {
    //     let js = r#"{ "value_i64": -1, "value_u64": 5, "value_f64": 1.2 }"#;
    //     b.iter(|| {
    //         let ap: ValuesToTest = serde_json::from_str(js).expect("Error de from str");
    //         test::black_box(ap);
    //     });
    // }

    // #[bench]
    // /// TODO: Why is this slower than `bench_de_string_from_value_to_struct`
    // fn bench_de_number_from_value_to_struct(b: &mut Bencher) {
    //     let js = r#"{ "value_i64": -1, "value_u64": 5, "value_f64": 1.2 }"#;
    //     b.iter(|| {
    //         let jv = serde_json::from_str(js).expect("Error de from str");
    //         let ap: ValuesToTest = serde_json::from_value(jv).expect("Error de from str");
    //         test::black_box(ap);
    //     });
    // }
}
