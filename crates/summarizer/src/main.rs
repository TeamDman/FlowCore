use clap::CommandFactory;
use clap::FromArgMatches;
use summarizer::Args;
use summarizer::run_program;
use tracing::info;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let mut cmd = Args::command();
    cmd = cmd.version(env!("CARGO_PKG_VERSION"));
    let args = Args::from_arg_matches(&cmd.get_matches())?;

    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(false)
        .with_max_level(match args.debug {
            true => tracing::level_filters::LevelFilter::DEBUG,
            false => tracing::level_filters::LevelFilter::INFO,
        })
        .init();

    info!("Starting summarizer");
    run_program(args).await?;
    Ok(())
}
