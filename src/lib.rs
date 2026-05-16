//! # json-streamparse-rs
//!
//! Streaming JSON balance detector. Feed bytes incrementally and ask
//! whether the buffer currently holds a complete top-level JSON value.
//!
//! This is *not* a full parser; it's the small utility you want when an
//! LLM is producing JSON token-by-token and you need to know "can I
//! hand this to `serde_json::from_str` yet?" without actually parsing
//! every prefix.
//!
//! String-aware (won't be fooled by `{` inside a string literal),
//! escape-aware (`\\\"` doesn't end the string).
//!
//! ## Example
//!
//! ```
//! use json_streamparse_rs::Balancer;
//! let mut b = Balancer::new();
//! b.push(b"{\"name\":\"Cl");
//! assert!(!b.complete());
//! b.push(b"aude\",\"v\":1}");
//! assert!(b.complete());
//! ```

#![deny(missing_docs)]

/// Streaming JSON balance detector.
#[derive(Debug, Default, Clone)]
pub struct Balancer {
    depth: i32,
    started: bool,
    in_string: bool,
    escape: bool,
    bytes_consumed: u64,
}

impl Balancer {
    /// Empty detector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed bytes. Updates internal state in place.
    pub fn push(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.bytes_consumed += 1;
            if self.in_string {
                if self.escape {
                    self.escape = false;
                } else if b == b'\\' {
                    self.escape = true;
                } else if b == b'"' {
                    self.in_string = false;
                }
                continue;
            }
            match b {
                b'{' | b'[' => {
                    self.depth += 1;
                    self.started = true;
                }
                b'}' | b']' => {
                    if self.depth > 0 {
                        self.depth -= 1;
                    }
                }
                b'"' => {
                    self.in_string = true;
                    self.started = true;
                }
                b' ' | b'\t' | b'\n' | b'\r' => {}
                _ => {
                    self.started = true;
                }
            }
        }
    }

    /// True when the input so far is non-empty and bracket-balanced
    /// (depth = 0) and not currently mid-string.
    pub fn complete(&self) -> bool {
        self.started && self.depth == 0 && !self.in_string
    }

    /// Current bracket depth (0 at the root).
    pub fn depth(&self) -> i32 {
        self.depth
    }

    /// Bytes consumed so far.
    pub fn bytes_consumed(&self) -> u64 {
        self.bytes_consumed
    }

    /// Reset to empty.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
