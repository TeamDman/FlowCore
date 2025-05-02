#[tokio::test]
pub async fn sky() -> eyre::Result<()> {
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
    let output_file = test_dir.join("o_sky_output.md");
    let _ = std::fs::remove_file(&output_file);
    
    let args = summarizer::Args {
        debug: false,
        command: summarizer::Commands::Ollama {
            model: "qwen3:1.7b".to_string(),
            input: test_dir.join("i_sky_input.md"),
            output: output_file,
        },
        stream: true,
    };

    let result = summarizer::run_program(args).await;
    let duration = start.elapsed();
    info!("Test completed in {:.2?}", duration);
    
    // Write timing information to file
    let timing_file = test_dir.join("t_sky_timing.txt");
    let mut file = File::create(timing_file)?;
    writeln!(file, "Test completed in {:.2?}", duration)?;
    
    result
}
