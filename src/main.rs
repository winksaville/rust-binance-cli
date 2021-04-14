use clap::{AppSettings, Clap};

#[derive(Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Cli {
    #[clap(short, long, env = "SECRET_KEY")]
    secret_key: String,

    #[clap(short, long, env = "API_KEY")]
    api_key: String,

    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() {
    let args = Cli::parse();

    #[allow(unused)]
    let sec_key: Vec<u8> = args.secret_key.as_bytes().to_vec();
    let api_key: Vec<u8> = args.api_key.as_bytes().to_vec();
    println!(
        "sec_key=secret key is never displayed api_key={}",
        std::str::from_utf8(&api_key).unwrap(),
    );
}
