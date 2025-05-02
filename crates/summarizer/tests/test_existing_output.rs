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