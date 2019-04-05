use std::collections::HashMap;
use std::process::Command;

use ivory::{ArrayKey, PhpVal};
use maplit::hashmap;
use pretty_assertions::assert_eq;
use std::fmt::Debug;

#[test]
fn zval_parsing() {
    let inputs: HashMap<&str, PhpVal> = hashmap! {
        "1" => PhpVal::Long(1),
        "1.1" => PhpVal::Double(1.1),
        "\"test\"" => PhpVal::String("test".into()),
        "true" => PhpVal::Bool(true),
        "false" => PhpVal::Bool(false),
        "null" => PhpVal::Null,
        "[1,2,3]" => vec![1, 2, 3].into(),
        "[1,2,\"foo\"]" => vec![
            PhpVal::Long(1),
            PhpVal::Long(2),
            PhpVal::String("foo".into())
        ].into(),
        "[1,2, 4 => 3]" => vec![
            (0u64, 1),
            (1, 2),
            (4, 3)
        ].into(),
        "[1,2, \"foo\" => 3]" => vec![
            (ArrayKey::from(0u64), 1),
            (ArrayKey::from(1u64), 2),
            (ArrayKey::from("foo".to_string()), 3)
        ].into(),
    };

    for (input, expected) in inputs {
        let code = format!("dump_arg({})", input);
        let result = run_php(&code).unwrap();
        assert_debug_eq(expected, &result);
    }
}

macro_rules! test_cast {
    ($name:ident, $method:expr, $in:expr, $fail:expr) => {
        #[test]
        fn $name() {
            let result = run_php(&format!("{}({})", $method, $in)).unwrap();
            assert_debug_eq($in, &result);
            assert_eq!(true, run_php(&format!("{}({})", $method, $fail)).is_err());
            assert_eq!(true, run_php(&format!("{}(null)", $method)).is_err());
            assert_eq!(true, run_php(&format!("{}()", $method)).is_err());
        }
    };
}

test_cast!(test_cast_long, "expect_long", 1, false);
test_cast!(test_cast_double, "expect_double", 1.1, false);
test_cast!(test_cast_string, "expect_string", "foo".to_string(), false);
test_cast!(test_cast_bool, "expect_bool", true, 17);

#[test]
fn test_cast_option() {
    let result = run_php("expect_option_bool(true)").unwrap();
    assert_debug_eq(Some(true), &result);
    assert_eq!(true, run_php("expect_option_bool(17)").is_err());
    let result = run_php("expect_option_bool(null)").unwrap();
    assert_debug_eq::<Option<bool>>(None, &result);
    let result = run_php("expect_option_bool()").unwrap();
    assert_debug_eq::<Option<bool>>(None, &result);
}

#[test]
fn test_imported() {
    assert_eq!("imported".to_string(), run_php("imported_fn()").unwrap());
}

/// Test that the result is the debug formatting of expected
fn assert_debug_eq<T: Debug>(expected: T, result: &str) {
    assert_eq!(format!("{:?}", expected), result);
}

/// Run some php code and return it's output
fn run_php(code: &str) -> Result<String, String> {
    let code = format!("{};", code);
    let output = Command::new("php")
        .args(&["-d", "extension=target/debug/libtests.so", "-r", &code])
        .output()
        .expect("Failed to run php script");
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).expect("invalid utf8"))
    } else {
        Err(String::from_utf8(output.stderr).expect("invalid utf8"))
    }
}
