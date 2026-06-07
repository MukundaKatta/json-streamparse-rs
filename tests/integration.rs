use json_streamparse_rs::Balancer;

#[test]
fn empty_is_not_complete() {
    let b = Balancer::new();
    assert!(!b.complete());
}

#[test]
fn full_object_one_push() {
    let mut b = Balancer::new();
    b.push(b"{\"a\":1}");
    assert!(b.complete());
    assert_eq!(b.depth(), 0);
}

#[test]
fn partial_object_not_complete() {
    let mut b = Balancer::new();
    b.push(b"{\"a\":");
    assert!(!b.complete());
    assert_eq!(b.depth(), 1);
}

#[test]
fn string_with_braces_doesnt_unbalance() {
    let mut b = Balancer::new();
    b.push(b"{\"text\":\"hello { world }\"}");
    assert!(b.complete());
}

#[test]
fn escape_sequences_dont_close_string_early() {
    let mut b = Balancer::new();
    b.push(b"{\"s\":\"foo \\\"bar\\\" baz\"}");
    assert!(b.complete());
}

#[test]
fn nested_brackets() {
    let mut b = Balancer::new();
    b.push(b"[1, [2, [3, [4]]]]");
    assert!(b.complete());
}

#[test]
fn streaming_byte_at_a_time() {
    let mut b = Balancer::new();
    let s = b"{\"foo\":[1,2,3]}";
    let mut completed_at = None;
    for (i, byte) in s.iter().enumerate() {
        b.push(&[*byte]);
        if b.complete() && completed_at.is_none() {
            completed_at = Some(i + 1);
        }
    }
    assert_eq!(completed_at, Some(s.len()));
}

#[test]
fn reset_clears_state() {
    let mut b = Balancer::new();
    b.push(b"{");
    b.reset();
    assert!(!b.complete());
    assert_eq!(b.depth(), 0);
    assert_eq!(b.bytes_consumed(), 0);
    assert!(!b.broken());
}

#[test]
fn extra_closing_bracket_is_broken_not_complete() {
    let mut b = Balancer::new();
    b.push(b"{}}");
    assert!(b.broken());
    assert!(!b.complete());
}

#[test]
fn stray_closing_bracket_is_broken() {
    let mut b = Balancer::new();
    b.push(b"}");
    assert!(b.broken());
    assert!(!b.complete());
}

#[test]
fn mismatched_bracket_types_are_broken() {
    let mut b = Balancer::new();
    b.push(b"{]");
    assert!(b.broken());
    assert!(!b.complete());
}

#[test]
fn broken_state_is_sticky_until_reset() {
    let mut b = Balancer::new();
    b.push(b"}"); // breaks it
    assert!(b.broken());
    b.push(b"{\"a\":1}"); // a valid value afterward must not "heal" it
    assert!(b.broken());
    assert!(!b.complete());
    b.reset();
    b.push(b"{\"a\":1}");
    assert!(!b.broken());
    assert!(b.complete());
}

#[test]
fn closing_bracket_inside_string_is_fine() {
    let mut b = Balancer::new();
    b.push(b"{\"k\":\"a]b}c\"}");
    assert!(!b.broken());
    assert!(b.complete());
}
