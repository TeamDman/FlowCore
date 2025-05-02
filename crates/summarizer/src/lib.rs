use clap::Parser;
use clap::Subcommand;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::info;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,

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
        input: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

pub async fn run_program(args: Args) -> eyre::Result<()> {
    match args.command {
        Commands::Ollama {
            model,
            input,
            output,
        } => {
            // Check if output file already exists
            let output_path = Path::new(&output);
            if output_path.exists() {
                return Err(eyre::eyre!("Output file already exists: {}", output));
            }

            // Read input file content
            let input_content = fs::read_to_string(&input)?;

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
            let request = GenerationRequest::new(model, input_content);

            // Generate response
            let response = ollama.generate(request).await?;

            // Write response to file
            file.write_all(response.response.as_bytes())?;
            file.flush()?;

            info!("Response written to {}", output);
        }
    }
    Ok(())
}
