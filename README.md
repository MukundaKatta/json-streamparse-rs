# json-streamparse-rs

[![crates.io](https://img.shields.io/crates/v/json-streamparse-rs.svg)](https://crates.io/crates/json-streamparse-rs)

Streaming JSON balance detector. "Can I hand this to `serde_json` yet?"
in O(1) per byte. String/escape-aware.

```rust
use json_streamparse_rs::Balancer;
let mut b = Balancer::new();
b.push(b"{\"name\":\"Cl");
assert!(!b.complete());
b.push(b"aude\",\"v\":1}");
assert!(b.complete());
```

Zero deps. MIT or Apache-2.0.
