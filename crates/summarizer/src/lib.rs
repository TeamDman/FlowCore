#![feature(async_fn_track_caller)]
use clap::Parser;
use clap::Subcommand;
use eyre::Context;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tokio_stream::StreamExt;
use tracing::info;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,

    #[arg(long, global = true, default_value = "false")]
    pub stream: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate a response using Ollama
    Ollama {
        /// The model to use
        #[arg(short, long, default_value = "llama2")]
        model: String,

        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate responses using multiple Ollama models
    MultiOllama {
        /// Comma-separated list of models to use
        #[arg(short, long)]
        models: Vec<String>,

        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Comma-separated list of output file paths
        #[arg(short, long)]
        outputs: Vec<PathBuf>,
    },
}

#[track_caller]
pub async fn run_program(args: Args) -> eyre::Result<()> {
    match run_program_inner(args).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.wrap_err(format!(
            "Failed to run program from {}",
            std::panic::Location::caller()
        ))),
    }
}

pub async fn run_program_inner(args: Args) -> eyre::Result<()> {
    match args.command {
        Commands::Ollama {
            model,
            input,
            output,
        } => {
            // Check if output file already exists
            let output_path = Path::new(&output);
            if matches!(tokio::fs::try_exists(output_path).await, Ok(true)) {
                return Err(eyre::eyre!(
                    "Output file already exists: {}",
                    output.display()
                ));
            }

            // Read input file content
            let input_content = fs::read_to_string(&input)
                .wrap_err(format!("Failed to read input file from {:?}", input.display()))?;

            // Ensure the output directory exists
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Open output file in append mode
            let mut file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(output_path)?;

            // Initialize Ollama client
            let ollama = Ollama::default();

            // Create generation request
            let messages = vec![ChatMessage::user(input_content)];
            let request = ChatMessageRequest::new(model.clone(), messages);

            if args.stream {
                // Stream the response
                let mut stream = ollama.send_chat_messages_stream(request).await?;
                while let Some(Ok(response)) = stream.next().await {
                    file.write_all(response.message.content.as_bytes())?;
                    file.flush()?;
                }
            } else {
                // Generate response in one go
                let response = ollama.send_chat_messages(request).await?;
                file.write_all(response.message.content.as_bytes())?;
                file.flush()?;
            }

            info!("Response written to {}", output.display());
        }
        Commands::MultiOllama {
            models,
            input,
            outputs,
        } => {
            if models.len() != outputs.len() {
                return Err(eyre::eyre!(
                    "Number of models ({}) must match number of output paths ({})",
                    models.len(),
                    outputs.len()
                ));
            }

            // Read input file content once
            let input_content = fs::read_to_string(&input)
                .wrap_err(format!("Failed to read input file from {:?}", input.display()))?;

            // Process each model-output pair
            for (model, output) in models.iter().zip(outputs.iter()) {
                let output_path = Path::new(output);
                
                // Check if output file already exists
                if matches!(tokio::fs::try_exists(output_path).await, Ok(true)) {
                    return Err(eyre::eyre!(
                        "Output file already exists: {}",
                        output.display()
                    ));
                }

                // Ensure the output directory exists
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Open output file in append mode
                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(output_path)?;

                // Initialize Ollama client
                let ollama = Ollama::default();

                // Create generation request
                let messages = vec![ChatMessage::user(input_content.clone())];
                let request = ChatMessageRequest::new(model.clone(), messages);

                if args.stream {
                    // Stream the response
                    let mut stream = ollama.send_chat_messages_stream(request).await?;
                    while let Some(Ok(response)) = stream.next().await {
                        file.write_all(response.message.content.as_bytes())?;
                        file.flush()?;
                    }
                } else {
                    // Generate response in one go
                    let response = ollama.send_chat_messages(request).await?;
                    file.write_all(response.message.content.as_bytes())?;
                    file.flush()?;
                }

                info!("Response written to {} using model {}", output.display(), model);
            }
        }
    }
    Ok(())
}
