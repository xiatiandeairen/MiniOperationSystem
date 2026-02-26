//! Environment variables — simple key-value store.

use minios_hal::println;
use spin::Mutex;

const MAX_VARS: usize = 16;
const MAX_KEY: usize = 32;
const MAX_VAL: usize = 64;

struct EnvVar {
    key: [u8; MAX_KEY],
    key_len: usize,
    val: [u8; MAX_VAL],
    val_len: usize,
}

struct Environment {
    vars: [Option<EnvVar>; MAX_VARS],
}

impl Environment {
    const fn new() -> Self {
        const NONE: Option<EnvVar> = None;
        Self {
            vars: [NONE; MAX_VARS],
        }
    }

    fn set(&mut self, key: &str, val: &str) {
        for v in self.vars.iter_mut().flatten() {
            if v.key[..v.key_len] == *key.as_bytes() {
                let vlen = val.len().min(MAX_VAL);
                v.val[..vlen].copy_from_slice(&val.as_bytes()[..vlen]);
                v.val_len = vlen;
                return;
            }
        }
        for slot in self.vars.iter_mut() {
            if slot.is_none() {
                let mut var = EnvVar {
                    key: [0; MAX_KEY],
                    key_len: 0,
                    val: [0; MAX_VAL],
                    val_len: 0,
                };
                let klen = key.len().min(MAX_KEY);
                var.key[..klen].copy_from_slice(&key.as_bytes()[..klen]);
                var.key_len = klen;
                let vlen = val.len().min(MAX_VAL);
                var.val[..vlen].copy_from_slice(&val.as_bytes()[..vlen]);
                var.val_len = vlen;
                *slot = Some(var);
                return;
            }
        }
        println!("env: too many variables (max {})", MAX_VARS);
    }
}

static ENV: Mutex<Environment> = Mutex::new(Environment::new());

/// Initialize default environment variables.
pub fn init_defaults() {
    let mut env = ENV.lock();
    env.set("VERSION", "0.4.0");
    env.set("SHELL", "minios-shell");
}

/// `set KEY VALUE` — set an environment variable.
pub fn cmd_set(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: set <key> <value>");
        return;
    }
    ENV.lock().set(args[0], args[1]);
    println!("{}={}", args[0], args[1]);
}

/// Looks up a variable and returns its value (or empty string).
pub fn get_var(key: &str) -> alloc::string::String {
    let env = ENV.lock();
    for v in env.vars.iter().flatten() {
        if v.key[..v.key_len] == *key.as_bytes() {
            let val = core::str::from_utf8(&v.val[..v.val_len]).unwrap_or("");
            return alloc::string::String::from(val);
        }
    }
    alloc::string::String::new()
}

/// `env` — list all environment variables.
pub fn cmd_env(_args: &[&str]) {
    let env = ENV.lock();
    for v in env.vars.iter().flatten() {
        let key = core::str::from_utf8(&v.key[..v.key_len]).unwrap_or("?");
        let val = core::str::from_utf8(&v.val[..v.val_len]).unwrap_or("?");
        println!("  {}={}", key, val);
    }
}
