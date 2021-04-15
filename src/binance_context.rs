pub struct BinanceContext {
    pub scheme: String,
    pub domain: String,
}

impl BinanceContext {
    pub fn new() -> Self {
        Self {
            scheme: "https".to_string(),
            domain: "binance.us".to_string(),
        }
    }

    pub fn make_url(&self, subdomain: &str, full_path: &str) -> String {
        let sd = if !subdomain.is_empty() {
            format!("{}.", subdomain)
        } else {
            "".to_string()
        };

        format!("{}://{}{}{}", self.scheme, sd, self.domain, full_path)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let ctx = BinanceContext::new();
        assert_eq!(ctx.scheme, "https");
        assert_eq!(ctx.domain, "binance.us");
    }

    #[test]
    fn test_make_url() {
        let ctx = BinanceContext::new();
        let url = ctx.make_url("api", "/api/v3/exchangeInfo");
        assert_eq!(url, "https://api.binance.us/api/v3/exchangeInfo");

        let url = ctx.make_url("", "/api/v3/exchangeInfo");
        assert_eq!(url, "https://binance.us/api/v3/exchangeInfo");
    }
}
