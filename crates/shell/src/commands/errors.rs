//! User-friendly error display with recovery suggestions.
use minios_hal::println;

/// Wraps error display with context and recovery advice.
pub fn show_error(cmd: &str, err: &str) {
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::RED);
    println!("Error in '{}': {}", cmd, err);
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
    if let Some(suggestion) = suggest_recovery(err) {
        println!("Suggestion: {}", suggestion);
    }
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
}

fn suggest_recovery(err: &str) -> Option<&'static str> {
    if err.contains("not found") {
        return Some("Check the path with 'ls'. File names are case-sensitive.");
    }
    if err.contains("not a directory") {
        return Some("The path points to a file, not a directory. Use 'cat' instead of 'ls'.");
    }
    if err.contains("not a file") {
        return Some("The path is a directory. Use 'ls' to view its contents.");
    }
    if err.contains("already exists") {
        return Some("A file or directory with that name exists. Choose a different name.");
    }
    if err.contains("permission") {
        return Some("This operation is not allowed on this file (e.g. writing to /proc/).");
    }
    if err.contains("invalid") {
        return Some("Check the command syntax with 'explain <command>'.");
    }
    if err.contains("Invalid PID") {
        return Some("Use 'ps' to see valid PIDs.");
    }
    None
}
