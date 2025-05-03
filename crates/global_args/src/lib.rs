use clap::Parser;

#[derive(Debug, Parser)]
pub struct GlobalArgs {
    /// Enable debug logging
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,

    /// Run in non-interactive mode (will fail if user interaction is needed)
    #[arg(long, global = true, default_value = "false")]
    pub non_interactive: bool,
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
