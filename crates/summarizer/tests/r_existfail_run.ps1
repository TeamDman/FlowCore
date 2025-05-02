#!/usr/bin/env pwsh

# Run the specific test with output capture disabled
cargo test -p summarizer test_existing_output_file -- --nocapture

# If you want to see the exit code, uncomment the following line:
# Write-Host "Exit code: $LASTEXITCODE" 