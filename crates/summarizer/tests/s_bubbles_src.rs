#[tokio::test]
pub async fn bubbles() -> eyre::Result<()> {
    use std::fs::File;
    use std::io::Write;
    use std::time::Instant;
    use tracing::info;

    // Initialize tracing for the test
    let _subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .compact()
        .init();

    let start = Instant::now();
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    // Create input file with question about bubbles
    let input_file = test_dir.join("i_bubbles_input.md");
    std::fs::write(&input_file, "Why do soap bubbles have rainbow colors?")?;

    // Clean up any existing output files
    let output_file1 = test_dir.join("o_bubbles_output1.md");
    let output_file2 = test_dir.join("o_bubbles_output2.md");
    let _ = std::fs::remove_file(&output_file1);
    let _ = std::fs::remove_file(&output_file2);

    let args = summarizer::Args {
        debug: false,
        command: summarizer::Commands::MultiOllama {
            models: vec!["qwen3:0.6b".to_string(), "qwen3:1.7b".to_string()],
            input: input_file,
            outputs: format!("{},{}", output_file1.display(), output_file2.display()),
        },
        stream: true,
    };

    let result = summarizer::run_program(args).await;
    let duration = start.elapsed();
    info!("Test completed in {:.2?}", duration);

    // Write timing information to file
    let timing_file = test_dir.join("t_bubbles_timing.txt");
    let mut file = File::create(timing_file)?;
    writeln!(file, "Test completed in {:.2?}", duration)?;

    result
}
