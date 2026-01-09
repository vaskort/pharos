use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    package: String,
}

fn main() {
    let cli = Cli::parse();
    println!("Hello bro {}", cli.package);
}
