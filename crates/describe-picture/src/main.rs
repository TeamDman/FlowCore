pub mod config;

use base64::Engine;
use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use config::DescribePictureConfig;
use eyre::bail;
use flow_core_config::IConfig;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::images::Image;
use std::fs;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,
    #[arg(long)]
    pub picture_path: PathBuf,
    #[arg(long)]
    pub prompt: String,
    #[arg(long)]
    pub model: Option<String>,
}

pub async fn get_image_model(args: &Args) -> eyre::Result<String> {
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

    let config = DescribePictureConfig::load().await?;
    let preferred_model = config.preferred_vision_model;
    if let Some(model) = preferred_model {
        debug!("Ensuring preferred model supports vision");
        let model_info = ollama.show_model_info(model.clone()).await?;
        if model_info.capabilities.contains(&"vision".to_string()) {
            return Ok(model);
        } else {
            bail!("Preferred model does not support vision: {}", model);
        }
    }

    debug!("No preferred model specified, prompting the user");
    let models = ollama.list_local_models().await?;
    let mut vision_candidates = Vec::new();
    for model in models {
        let model_info = ollama.show_model_info("gemma3:27b".to_string()).await?;
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

    debug!("Ensuring picture path exists");
    if !args.picture_path.exists() {
        bail!(
            "Picture path does not exist: {}",
            args.picture_path.display()
        );
    }

    debug!("Identifying model");
    let model = get_image_model(&args).await?;

    debug!("Reading picture");
    let image_bytes = fs::read(&args.picture_path)?;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    let image = Image::from_base64(&base64_image);

    debug!("Creating Ollama request");
    let request = GenerationRequest::new(model, args.prompt).add_image(image);

    debug!("Sending request to Ollama");
    let ollama = Ollama::default();
    let response = ollama.generate(request).await?;

    println!("{}", response.response);

    Ok(())
}
