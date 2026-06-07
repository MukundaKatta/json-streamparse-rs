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
//! escape-aware (`\\\"` doesn't end the string), and bracket-type aware
//! (`{...]` and stray closers like `}` are reported as broken, not complete).
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
    /// Stack of currently-open bracket bytes (`b'{'` or `b'['`).
    /// Its length is the current nesting depth.
    open: Vec<u8>,
    started: bool,
    in_string: bool,
    escape: bool,
    /// Set once the input is structurally broken (a closing bracket with
    /// nothing open, or a closer that does not match the most recent opener).
    /// Once broken, the buffer can never be `complete()` again until `reset()`.
    broken: bool,
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
                    self.open.push(b);
                    self.started = true;
                }
                b'}' | b']' => {
                    let want = if b == b'}' { b'{' } else { b'[' };
                    match self.open.pop() {
                        // Closer matches the most recent opener: good.
                        Some(opener) if opener == want => {}
                        // Wrong closer type, or a closer with nothing open:
                        // the structure is malformed.
                        _ => self.broken = true,
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

    /// True when the input so far is non-empty, structurally sound, fully
    /// bracket-balanced (depth = 0) and not currently mid-string.
    ///
    /// Returns `false` if the input is [`broken`](Self::broken) (e.g. an
    /// unmatched closing bracket or a mismatched bracket pair like `{...]`).
    pub fn complete(&self) -> bool {
        self.started && !self.broken && self.open.is_empty() && !self.in_string
    }

    /// True if the input is structurally broken: a closing bracket appeared
    /// with nothing open, or a closer did not match its opener (e.g. `{]`).
    /// A broken buffer can never become [`complete`](Self::complete) again
    /// until [`reset`](Self::reset).
    pub fn broken(&self) -> bool {
        self.broken
    }

    /// Current bracket depth (0 at the root).
    pub fn depth(&self) -> i32 {
        self.open.len() as i32
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
