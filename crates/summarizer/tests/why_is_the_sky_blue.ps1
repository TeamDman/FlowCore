#!/usr/bin/env pwsh

# Run the specific test with output capture disabled
cargo test -p summarizer why_is_the_sky_blue -- --nocapture

# If you want to see the exit code, uncomment the following line:
# Write-Host "Exit code: $LASTEXITCODE" 