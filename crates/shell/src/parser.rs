//! Command-line parser for the shell.

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
