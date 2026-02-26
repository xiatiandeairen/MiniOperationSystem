//! Text processing commands: head, grep, wc.

extern crate alloc;

use minios_common::traits::fs::FileSystem;
use minios_common::types::OpenFlags;
use minios_hal::println;

/// Shows the first N lines of a file (default: 10).
pub fn cmd_head(args: &[&str]) {
    let (count, path) = parse_head_args(args);
    let content = match read_file_content(path) {
        Some(c) => c,
        None => return,
    };
    for (i, line) in content.lines().enumerate() {
        if i >= count {
            break;
        }
        println!("{}", line);
    }
}

fn parse_head_args<'a>(args: &[&'a str]) -> (usize, &'a str) {
    match args.len() {
        0 => {
            println!("Usage: head [N] <file>");
            (10, "")
        }
        1 => (10, args[0]),
        _ => {
            let n = parse_usize(args[0]).unwrap_or(10);
            (n, args[1])
        }
    }
}

/// Prints lines containing a pattern.
pub fn cmd_grep(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: grep <pattern> <file>");
        return;
    }
    let pattern = args[0];
    let path = args[1];
    let content = match read_file_content(path) {
        Some(c) => c,
        None => return,
    };
    let mut found = 0;
    for line in content.lines() {
        if contains_pattern(line, pattern) {
            println!("{}", line);
            found += 1;
        }
    }
    if found == 0 {
        println!("(no matches)");
    }
    super::journey::mark(super::journey::STEP_GREP);
}

/// Simple substring search (case-sensitive).
fn contains_pattern(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    for i in 0..=(haystack.len() - needle.len()) {
        if &haystack.as_bytes()[i..i + needle.len()] == needle.as_bytes() {
            return true;
        }
    }
    false
}

/// Counts lines, words, and bytes in a file.
pub fn cmd_wc(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: wc <file>");
        return;
    }
    let path = args[0];
    let content = match read_file_content(path) {
        Some(c) => c,
        None => return,
    };
    let bytes = content.len();
    let lines = content.lines().count();
    let words = content.split_whitespace().count();
    println!("  {:>6} {:>6} {:>6}  {}", lines, words, bytes, path);
}

/// Reads a file's entire content as a string.
fn read_file_content(path: &str) -> Option<alloc::string::String> {
    if path.is_empty() {
        return None;
    }
    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("Filesystem not initialized");
            return None;
        }
    };
    let fd = match vfs.open(path, OpenFlags::READ) {
        Ok(fd) => fd,
        Err(e) => {
            println!("{}: {}", path, e);
            return None;
        }
    };
    let mut buf = [0u8; 4096];
    let n = vfs.read(fd, &mut buf).unwrap_or(0);
    vfs.close(fd).ok();
    drop(vfs_guard);
    core::str::from_utf8(&buf[..n])
        .ok()
        .map(alloc::string::String::from)
}

fn parse_usize(s: &str) -> Option<usize> {
    let mut result: usize = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_exact_match() {
        assert!(contains_pattern("hello world", "world"));
    }

    #[test]
    fn contains_at_start() {
        assert!(contains_pattern("hello", "hel"));
    }

    #[test]
    fn contains_not_found() {
        assert!(!contains_pattern("hello", "xyz"));
    }

    #[test]
    fn contains_empty_pattern() {
        assert!(contains_pattern("hello", ""));
    }

    #[test]
    fn contains_pattern_longer_than_haystack() {
        assert!(!contains_pattern("hi", "hello"));
    }
}
