use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use flow_core_global_args::GlobalArgs;
use tracing::info;

#[derive(Debug, Parser)]
#[command(name = "fc", bin_name = "fc")]
pub struct Args {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Start the GUI application
    #[command(name = "gui")]
    Gui(GuiCommand),

    /// Describe images using AI
    #[command(name = "image")]
    Image(ImageCommand),
}
impl Command {
    pub async fn handle(self, global: &GlobalArgs) -> eyre::Result<()> {
        match self {
            Self::Gui(cmd) => cmd.handle(global).await?,
            Self::Image(cmd) => cmd.handle(global).await?,
        };
        Ok(())
    }
}
#[derive(Debug, Parser)]
pub struct GuiCommand {
    /// Start the GUI application
    #[command(subcommand)]
    command: GuiSubcommand,
}
impl GuiCommand {
    pub async fn handle(self, _global: &GlobalArgs) -> eyre::Result<()> {
        match self.command {
            GuiSubcommand::Start => {
                info!("Starting GUI application");
            }
        };
        Ok(())
    }
}
#[derive(Debug, Parser)]
pub enum GuiSubcommand {
    /// Start the GUI application
    Start,
}

#[derive(Debug, Parser)]
pub struct ImageCommand {
    /// Describe images using AI
    #[command(subcommand)]
    command: flow_core_describe_picture::Command,
}
impl ImageCommand {
    pub async fn handle(self, global: &GlobalArgs) -> eyre::Result<()> {
        match self.command {
            flow_core_describe_picture::Command::Describe(cmd) => {
                cmd.handle(global).await?;
            }
            flow_core_describe_picture::Command::Config(cmd) => {
                cmd.handle(global).await?;
            }
        };
        Ok(())
    }
}

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
        .with_max_level(match args.global.debug {
            true => tracing::level_filters::LevelFilter::DEBUG,
            false => tracing::level_filters::LevelFilter::INFO,
        })
        .init();

    args.command.handle(&args.global).await?;

    Ok(())
}
