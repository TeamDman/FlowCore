pub mod config;

use base64::Engine;
use clap::Parser;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use config::DescribePictureConfig;
use eyre::Context;
use eyre::bail;
use flow_core_config::IConfig;
use flow_core_global_args::GlobalArgs;
use holda::Holda;
use holda::StringHolda;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::images::Image;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::task::JoinSet;
use tracing::debug;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Describe a single image
    Describe(DescribeCommand),
    /// Manage configuration
    Config(ConfigCommand),
}
impl Command {
    pub async fn handle(self, global: &GlobalArgs) -> eyre::Result<()> {
        match self {
            Self::Describe(cmd) => {
                let results = cmd.handle(global).await?;
                for (path, description) in results {
                    println!("{}: {}", path, description.inner);
                }
            }
            Self::Config(cmd) => {
                cmd.handle(global).await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Debug, Parser)]
pub enum ConfigSubcommand {
    /// Show current configuration
    Show,
    /// List configuration file paths
    List,
    /// Set a configuration value
    Set(SetCommand),
    /// Unset a configuration value
    Unset(UnsetCommand),
    /// Reset configuration to defaults
    Reset,
}

impl ConfigCommand {
    pub async fn handle(self, global: &GlobalArgs) -> eyre::Result<()> {
        match self.command {
            ConfigSubcommand::Show => {
                let config = DescribePictureConfig::load().await?;
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
            ConfigSubcommand::List => {
                let path = DescribePictureConfig::config_path();
                println!("{}", path.display());
            }
            ConfigSubcommand::Set(cmd) => {
                let mut config = DescribePictureConfig::load().await?;
                cmd.apply(&mut config)?;
                config.save().await?;
            }
            ConfigSubcommand::Unset(cmd) => {
                let mut config = DescribePictureConfig::load().await?;
                cmd.apply(&mut config)?;
                config.save().await?;
            }
            ConfigSubcommand::Reset => {
                if !global.non_interactive {
                    if !cloud_terrastodon_user_input::are_you_sure(
                        "Are you sure you want to reset the configuration to defaults?",
                    )? {
                        bail!("User did not confirm");
                    }
                }
                let config = DescribePictureConfig::default();
                config.save().await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct SetCommand {
    /// The configuration key to set
    #[arg(value_enum)]
    pub key: ConfigKey,
    /// The value to set
    pub value: String,
}

impl SetCommand {
    fn apply(&self, config: &mut DescribePictureConfig) -> eyre::Result<()> {
        match self.key {
            ConfigKey::PreferredVisionModel => {
                config.preferred_vision_model = Some(self.value.clone());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct UnsetCommand {
    /// The configuration key to unset
    #[arg(value_enum)]
    pub key: ConfigKey,
}

impl UnsetCommand {
    fn apply(&self, config: &mut DescribePictureConfig) -> eyre::Result<()> {
        match self.key {
            ConfigKey::PreferredVisionModel => {
                config.preferred_vision_model = None;
            }
        }
        Ok(())
    }
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum ConfigKey {
    PreferredVisionModel,
}

#[derive(Debug, Parser)]
pub struct DescribeCommand {
    /// Path to the image to describe
    #[arg(short = 'i', long = "image-path", required = true)]
    pub image_path: Vec<ImagePath>,

    /// Optional prompt/instructions for the model
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Optional model name to use
    #[arg(short, long)]
    pub model: Option<String>,
}

impl DescribeCommand {
    pub async fn handle(
        self,
        global: &GlobalArgs,
    ) -> eyre::Result<Vec<(ImagePath, ImageDescription)>> {
        debug!("Ensuring picture path exists");
        let mut rtn = Vec::new();
        for image_path in self.image_path.iter() {
            if !matches!(tokio::fs::try_exists(image_path.as_path()).await, Ok(true)) {
                bail!("Picture path does not exist: {}", image_path.display());
            }
        }

        debug!("Identifying model");
        let model = get_image_model(&self, global).await?;

        let prompt = self
            .prompt
            .unwrap_or_else(|| "Describe this image".to_string());
        let mut join_set = JoinSet::<eyre::Result<(ImagePath, ImageDescription)>>::new();
        for image_path in self.image_path {
            let prompt = prompt.to_owned();
            let model = model.to_owned();
            join_set.spawn(async move {
                debug!("Reading picture");
                let image_bytes = fs::read(image_path.as_path())?;
                let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
                let image = Image::from_base64(&base64_image);

                debug!("Creating Ollama request");
                let request = GenerationRequest::new(model, prompt).add_image(image);

                debug!("Sending request to Ollama");
                let ollama = Ollama::default();
                let response = ollama.generate(request).await?;

                Ok((image_path, response.response.into()))
            });
        }
        while let Some(result) = join_set.join_next().await {
            debug!("Received result, {} remain...", join_set.len());
            rtn.push(result??);
        }
        Ok(rtn)
    }
}

pub async fn get_image_model(args: &DescribeCommand, global: &GlobalArgs) -> eyre::Result<String> {
    let ollama = Ollama::default();
    if let Some(model) = &args.model {
        debug!("Ensuring model passed in args supports vision");
        let model_info = ollama.show_model_info(model.clone()).await?;
        if model_info.capabilities.contains(&"vision".to_string()) {
            return Ok(model.clone());
        } else {
            bail!("Model passed in args does not support vision: {}", model);
        }
    }

    let mut config = DescribePictureConfig::load().await?;
    let preferred_model = config.preferred_vision_model;
    if let Some(model) = preferred_model {
        debug!("Ensuring preferred model supports vision");
        let model_info = ollama.show_model_info(model.clone()).await?;
        if model_info.capabilities.contains(&"vision".to_string()) {
            return Ok(model);
        } else {
            config.preferred_vision_model = None;
            config.save().await?;
            bail!("Preferred model does not support vision: {}", model);
        }
    }

    if global.non_interactive {
        bail!("No model specified and non-interactive mode is enabled");
    }

    debug!("No preferred model specified, prompting the user");
    let models = ollama.list_local_models().await?;
    let mut vision_candidates = Vec::new();
    for model in models {
        let model_info = ollama
            .show_model_info(model.name.to_owned())
            .await
            .wrap_err(format!("Failed to get model info for {}", model.name))?;
        if model_info.capabilities.contains(&"vision".to_string()) {
            vision_candidates.push(model);
        }
    }

    if vision_candidates.is_empty() {
        bail!("No models support vision");
    }

    let chosen = pick(FzfArgs {
        choices: vision_candidates
            .into_iter()
            .map(|c| Choice {
                key: c.name.clone(),
                value: c,
            })
            .collect(),
        header: Some("Choose a model".to_string()),
        ..Default::default()
    })?;
    Ok(chosen.value.name)
}

#[derive(StringHolda)]
pub struct ImageDescription {
    pub inner: String,
}

#[derive(Holda)]
#[holda(NoDisplay)]
pub struct ImagePath {
    pub inner: PathBuf,
}
impl std::fmt::Display for ImagePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}
impl FromStr for ImagePath {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ImagePath {
            inner: PathBuf::from(s),
        })
    }
}
