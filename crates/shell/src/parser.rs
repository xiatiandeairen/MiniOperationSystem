//! Minimal command-line parser for `no_std` environments.
//!
//! Splits an input line into command name + arguments by whitespace.
//! Supports up to 16 tokens. No quoting or escaping.
//!
//! This parser is a self-contained, allocation-free component that
//! could be extracted into its own crate for embedded shells.

/// Maximum number of arguments (including the command name).
const MAX_ARGS: usize = 16;

/// Parsed command: the command name and its arguments.
pub struct ParsedCommand<'a> {
    tokens: [&'a str; MAX_ARGS],
    count: usize,
}

impl<'a> ParsedCommand<'a> {
    /// Returns the command name, or an empty string if no input.
    pub fn command(&self) -> &'a str {
        if self.count > 0 {
            self.tokens[0]
        } else {
            ""
        }
    }

    /// Returns the argument slice (excluding the command name).
    pub fn args(&self) -> &[&'a str] {
        if self.count > 1 {
            &self.tokens[1..self.count]
        } else {
            &[]
        }
    }

    /// Returns `true` if the input line was empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Splits a line by whitespace into command + arguments.
pub fn parse(line: &str) -> ParsedCommand<'_> {
    let mut result = ParsedCommand {
        tokens: [""; MAX_ARGS],
        count: 0,
    };

    for token in line.split_whitespace() {
        if result.count >= MAX_ARGS {
            break;
        }
        result.tokens[result.count] = token;
        result.count += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let p = parse("");
        assert!(p.is_empty());
    }

    #[test]
    fn parse_whitespace_only() {
        let p = parse("   ");
        assert!(p.is_empty());
    }

    #[test]
    fn parse_single_command() {
        let p = parse("help");
        assert_eq!(p.command(), "help");
        assert_eq!(p.args().len(), 0);
    }

    #[test]
    fn parse_command_with_args() {
        let p = parse("echo hello world");
        assert_eq!(p.command(), "echo");
        assert_eq!(p.args(), &["hello", "world"]);
    }

    #[test]
    fn parse_extra_whitespace() {
        let p = parse("  ls   /tmp  ");
        assert_eq!(p.command(), "ls");
        assert_eq!(p.args(), &["/tmp"]);
    }

    #[test]
    fn parse_many_args() {
        let p = parse("a b c d e f g h i j k l m n o p");
        assert_eq!(p.command(), "a");
        assert_eq!(p.args().len(), 15);
    }
}
