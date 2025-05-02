#[tokio::test]
pub async fn existfail() -> eyre::Result<()> {
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let output_file = test_dir.join("o_sky_output.md");
    
    // Create the output file first
    std::fs::write(&output_file, "This file already exists")?;
    
    let args = summarizer::Args {
        debug: false,
        stream: false,
        command: summarizer::Commands::Ollama {
            model: "qwen3:1.7b".to_string(),
            input: test_dir.join("s_sky_src.rs"),
            output: output_file,
        },
    };

    // Run the program and expect it to fail
    let result = summarizer::run_program(args).await;
    assert!(result.is_err(), "Expected error when output file exists");
        
    Ok(())
} 