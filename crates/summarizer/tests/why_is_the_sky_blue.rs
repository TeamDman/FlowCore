#[tokio::test]
pub async fn why_is_the_sky_blue() -> eyre::Result<()> {
    use std::time::Instant;
    use tracing::info;
    use std::fs::File;
    use std::io::Write;

    // Initialize tracing for the test
    let _subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .compact()
        .init();

    let start = Instant::now();
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    
    // Clean up any existing output files
    let output_file = test_dir.join("why_is_the_sky_blue_output.md");
    let _ = std::fs::remove_file(&output_file);
    
    let args = summarizer::Args {
        debug: false,
        command: summarizer::Commands::Ollama {
            model: "qwen3:1.7b".to_string(),
            input: test_dir.join("why_is_the_sky_blue_input.md").to_string_lossy().into_owned(),
            output: output_file.to_string_lossy().into_owned(),
        },
    };

    let result = summarizer::run_program(args).await;
    let duration = start.elapsed();
    info!("Test completed in {:.2?}", duration);
    
    // Write timing information to file
    let timing_file = test_dir.join("why_is_the_sky_blue_timing.txt");
    let mut file = File::create(timing_file)?;
    writeln!(file, "Test completed in {:.2?}", duration)?;
    
    result
}

#[tokio::test]
pub async fn test_existing_output_file() -> eyre::Result<()> {
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let output_file = test_dir.join("test_existing_output.md");
    
    // Create the output file first
    std::fs::write(&output_file, "This file already exists")?;
    
    let args = summarizer::Args {
        debug: false,
        command: summarizer::Commands::Ollama {
            model: "qwen3:1.7b".to_string(),
            input: test_dir.join("why_is_the_sky_blue_input.md").to_string_lossy().into_owned(),
            output: output_file.to_string_lossy().into_owned(),
        },
    };

    // Run the program and expect it to fail
    let result = summarizer::run_program(args).await;
    assert!(result.is_err(), "Expected error when output file exists");
    
    // Verify the error message
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Output file already exists"), 
            "Expected error message about existing file");
    
    Ok(())
}
