mod config;
mod merger;

use merger::Merger;

fn main() {
    let merger = Merger::from_config_file("config.toml");
    println!("{:?}", merger);
}
