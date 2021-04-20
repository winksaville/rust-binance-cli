#[allow(unused)]
use serde::{de, de::SeqAccess, de::Error, de::Visitor, Deserialize, Deserializer, Serialize};
use serde_json::value::Value;

use std::collections::HashMap;

use crate::exchange_info::Symbol;

#[derive(Debug, Deserialize, Serialize)]
pub struct SymbolX {
    pub symbol: String,
    pub value: i64,
}

impl Default for SymbolX {
    fn default() -> Self {
        SymbolX {
            symbol: "".to_string(),
            value: 0,
        }
    }
}
impl SymbolX {
    fn new(symbol: String, value: i64) -> SymbolX {
        SymbolX {
            symbol,
            value
        }
    }

    fn new_boxed(symbol: String, value: i64) -> Box<SymbolX> {
        Box::new(SymbolX {
            symbol,
            value
        })
    }
}

fn to_symbolx(item: Value) -> Option<SymbolX> {
    let obj = item.as_object()?;
    let mut sx = SymbolX::default();
    sx.symbol = obj.get("symbol")?.to_string();
    sx.value = obj.get("value")?.as_i64()?;
    Some(sx)
}

// Convert a string or number to i64
//pub fn de_vec_to_hashmap<'de, D: Deserializer<'de>>(
//    deserializer: D,
//) -> Result<HashMap<String, Box<SymbolX>>, D::Error> {
//    let first = Value::deserialize(deserializer);
//    println!("first={:#?}", first);
//    Ok(match first? {
//        Value::Array(a) => {
//            println!("de_vec_to_hashmap: a={:#?}", a);
//            let mut hm = HashMap::<String, Box<SymbolX>>::new();
//            for item in a {
//                let sx = to_symbolx(item);
//                match sx {
//                    Some(sx) => {
//                        let key = sx.symbol.clone();
//                        hm.insert(key, sx);
//                    }
//                    None => {
//                        return Err(de::Error::custom("Not a SymbolX object"));
//                    }
//                }
//            }
//            hm
//        }
//        _ => {
//            return Err(de::Error::custom("Expecting Array"));
//        }
//    })
//}

// from: https://github.com/serde-rs/serde/issues/936#ref-issue-557235055
pub fn de_vec_to_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, SymbolX>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = HashMap<String, SymbolX>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, SymbolX>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<String, SymbolX> =
                HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<SymbolX>()? {
                // println!("item={:#?}", item);
                map.insert(item.symbol.clone(), item);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test::Bencher;
    // use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct ValuesToTest {
        #[serde(deserialize_with = "de_vec_to_hashmap")]
        symbols : HashMap<String, SymbolX>,
    }

    #[test]
    fn test_de_vec_to_hashmap() {
        let js = r#"{ "symbols": [ { "symbol": "v1", "value": -1 }, { "symbol": "v2", "value": 0 }, { "symbol": "v3", "value": 1 } ] } "#;
        let ds: ValuesToTest = serde_json::from_str(js).expect("Error de_vec_map");
        println!("ds={:#?}", ds);
        match ds.symbols.get("v1") {
            Some(sx) => assert_eq!(sx.value, -1i64),
            None => assert!(false, "v1 not found"),
        }
        match ds.symbols.get("v2") {
            Some(sx) => assert_eq!(sx.value, 0i64),
            None => assert!(false, "v2 not found"),
        }
        match ds.symbols.get("v3") {
            Some(sx) => assert_eq!(sx.value, 1i64),
            None => assert!(false, "v3 not found"),
        }
    }
}
