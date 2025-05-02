use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use clap::Subcommand;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::debug;
use tracing::info;

pub fn run_program(args: Args) -> eyre::Result<()> {
    match args.command {
        Commands::Ollama {
            model,
            input,
            output,
        } => {
            // Read input file content
            let input_content = fs::read_to_string(&input)?;
            let output_path = Path::new(&output);

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
