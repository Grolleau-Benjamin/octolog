use clap::Parser;

#[tokio::main]
async fn main() {
    let args = octolog::cli::CliArgs::parse();

    if let Err(err) = octolog::app::run(args).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
