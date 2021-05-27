use vergen::{vergen, Config, ShaKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Short;

    // Do `cargo build -vv` to see the output
    // println!("build.rs: config: {:#?}", config);

    Ok(vergen(config)?)
}
