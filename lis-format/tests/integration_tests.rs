//! Integration tests for lis-format

use lis_format::{format, format_with_config, is_formatted, FormatConfig, IndentStyle};

#[test]
fn test_format_empty_function() {
    let input = "fn main(){}";
    let result = format(input).unwrap();
    assert!(result.contains("fn main() {"));
    assert!(result.contains("\n}"));
}

#[test]
fn test_format_function_with_statements() {
    let input = "fn main(){let x=42;let y=x+10;return y;}";
    let result = format(input).unwrap();

    assert!(result.contains("fn main() {"));
    assert!(result.contains("let x = 42;"));
    assert!(result.contains("let y = x + 10;"));
    assert!(result.contains("return y;"));
}

#[test]
fn test_format_preserves_semantics() {
    let input = r#"
fn add(a:Int, b:Int) {
    return a + b;
}
"#;
    let result = format(input).unwrap();

    // Should contain same structure
    assert!(result.contains("fn add"));
    assert!(result.contains("a: Int"));
    assert!(result.contains("b: Int"));
    assert!(result.contains("return a + b;"));
}

#[test]
fn test_format_with_if_statement() {
    let input = "fn test(){if x>0{return x;}else{return 0;}}";
    let result = format(input).unwrap();

    assert!(result.contains("if x > 0 {"));
    assert!(result.contains("} else {"));
    assert!(result.contains("return x;"));
    assert!(result.contains("return 0;"));
}

#[test]
fn test_format_loop() {
    let input = "fn test(){loop{break;}}";
    let result = format(input).unwrap();

    assert!(result.contains("loop {"));
    assert!(result.contains("break;"));
}

#[test]
fn test_format_pipe_operator() {
    let input = "fn test(){let x=input|>transform1|>transform2;}";
    let result = format(input).unwrap();

    assert!(result.contains("input |> transform1 |> transform2"));
}

#[test]
fn test_format_state_construct() {
    let input = "fn test(){let s=State{L0:x,L1:y,L2:z,L3:w,L4:a,L5:b,L6:c,L7:d,L8:e,L9:f,LA:g,LB:h,LC:i,LD:j,LE:k,LF:l};}";
    let result = format(input).unwrap();

    assert!(result.contains("State {"));
    assert!(result.contains("L0:"));
    assert!(result.contains("LF:"));
}

#[test]
fn test_format_layer_access() {
    let input = "fn test(){let x=state.L0+state.L1;}";
    let result = format(input).unwrap();

    assert!(result.contains("state.L0"));
    assert!(result.contains("state.L1"));
}

#[test]
fn test_format_complex_expr() {
    let input = "fn test(){let x=(1+2)*3/4;}";
    let result = format(input).unwrap();

    assert!(result.contains("(1 + 2)"));
    assert!(result.contains("* 3 / 4"));
}

#[test]
fn test_format_feedback() {
    let input = "fn test(){let x=feedback y*2;}";
    let result = format(input).unwrap();

    assert!(result.contains("feedback"));
}

#[test]
fn test_format_emerge() {
    let input = "fn test(){let x=emerge state;}";
    let result = format(input).unwrap();

    assert!(result.contains("emerge"));
}

#[test]
fn test_format_transform() {
    let input = "transform normalize(s:State){return s;}";
    let result = format(input).unwrap();

    assert!(result.contains("transform normalize"));
}

#[test]
fn test_format_type_alias() {
    let input = "type MyState=State;";
    let result = format(input).unwrap();

    assert!(result.contains("type MyState = State;"));
}

#[test]
fn test_compact_format() {
    let config = FormatConfig::compact();
    let input = "fn main() { let x = 42; }";
    let result = format_with_config(input, &config).unwrap();

    // Compact style has no spaces
    assert!(result.contains("fn main(){"));
    assert!(result.contains("let x=42;"));
}

#[test]
fn test_readable_format() {
    let config = FormatConfig::readable();
    let input = "fn a(){}fn b(){}";
    let result = format_with_config(input, &config).unwrap();

    // Readable style has extra blank lines
    let lines: Vec<&str> = result.lines().collect();
    let blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count();
    assert!(blank_lines >= 2);
}

#[test]
fn test_custom_indent_size() {
    let config = FormatConfig {
        indent_style: IndentStyle::Spaces(2),
        ..Default::default()
    };

    let input = "fn main(){let x=42;}";
    let result = format_with_config(input, &config).unwrap();

    // Check for 2-space indentation
    assert!(result.contains("\n  let x = 42;"));
}

#[test]
fn test_tabs_indent() {
    let config = FormatConfig {
        indent_style: IndentStyle::Tabs,
        ..Default::default()
    };

    let input = "fn main(){let x=42;}";
    let result = format_with_config(input, &config).unwrap();

    // Check for tab indentation
    assert!(result.contains("\n\tlet x = 42;"));
}

#[test]
fn test_is_formatted_true() {
    let formatted = r#"fn main() {
    let x = 42;
}
"#;
    assert!(is_formatted(formatted).unwrap());
}

#[test]
fn test_is_formatted_false() {
    let unformatted = "fn main(){let x=42;}";
    assert!(!is_formatted(unformatted).unwrap());
}

#[test]
fn test_format_idempotent() {
    let input = "fn main(){let x=42;}";
    let formatted1 = format(input).unwrap();
    let formatted2 = format(&formatted1).unwrap();

    assert_eq!(formatted1, formatted2, "Formatting should be idempotent");
}

#[test]
fn test_multiple_functions() {
    let input = "fn a(){}fn b(){}fn c(){}";
    let result = format(input).unwrap();

    // Should have blank lines between functions
    assert!(result.contains("fn a()"));
    assert!(result.contains("fn b()"));
    assert!(result.contains("fn c()"));
}

#[test]
fn test_nested_blocks() {
    let input = "fn test(){if x{if y{let z=0;}}}";
    let result = format(input).unwrap();

    // Check nested indentation exists
    assert!(result.contains("    if x {"));
    assert!(result.contains("        if y {"));
}

#[test]
fn test_function_call_with_args() {
    let input = "fn test(){call(a,b,c);}";
    let result = format(input).unwrap();

    assert!(result.contains("call(a, b, c)"));
}

#[test]
fn test_binary_operators() {
    let operators = vec![
        ("x+y", "x + y"),
        ("x-y", "x - y"),
        ("x*y", "x * y"),
        ("x/y", "x / y"),
        ("x**y", "x ** y"),
        ("x==y", "x == y"),
        ("x!=y", "x != y"),
        ("x<y", "x < y"),
        ("x<=y", "x <= y"),
        ("x>y", "x > y"),
        ("x>=y", "x >= y"),
        ("x&&y", "x && y"),
        ("x||y", "x || y"),
    ];

    for (input_expr, expected_expr) in operators {
        let input = format!("fn test(){{let z={};}}", input_expr);
        let result = format(&input).unwrap();
        assert!(result.contains(expected_expr),
                "Expected '{}' to be formatted as '{}'", input_expr, expected_expr);
    }
}

#[test]
fn test_string_literals() {
    let input = r#"fn test(){let s="hello world";}"#;
    let result = format(input).unwrap();

    assert!(result.contains(r#""hello world""#));
}

#[test]
fn test_bool_literals() {
    let input = "fn test(){let a=true;let b=false;}";
    let result = format(input).unwrap();

    assert!(result.contains("true"));
    assert!(result.contains("false"));
}

#[test]
fn test_float_literals() {
    let input = "fn test(){let x=3.14;}";
    let result = format(input).unwrap();

    assert!(result.contains("3.14"));
}
