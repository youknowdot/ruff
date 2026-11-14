use ruff_python_ast::PythonVersion;
use ruff_python_parser::{Mode, ParseOptions, parse_unchecked};

#[test]
fn specific_diagnostics() {
    for (source, message) in [
        (
            "assert x = 1",
            "cannot assign to name here. Maybe you meant '==' instead of '='?",
        ),
        (
            "assert False, x = 1",
            "cannot assign to name here. Maybe you meant '==' instead of '='?",
        ),
        (
            "assert x := 1",
            "cannot use named expression without parentheses here",
        ),
        (
            "case 1:\n    pass",
            "case statement must be inside match statement",
        ),
        ("case 1:", "expected an indented block"),
        (
            "case 1:\n    raise from None",
            "did you forget an expression between 'raise' and 'from'?",
        ),
        (
            "case {_.x: value}:\n    pass",
            "case statement must be inside match statement",
        ),
        (
            "value = f\"\"\"{\n1\n\"\"\"",
            "f-string: expecting '}' to close '{' on line 1",
        ),
        (
            "value = t\"\"\"{\n1\n\"\"\"",
            "t-string: expecting '}' to close '{' on line 1",
        ),
        (
            "value = f\"\"\"{0:\n{1\n\"\"\"",
            "f-string: expecting '}' to close '{' on line 2",
        ),
    ] {
        let parsed = parse_unchecked(
            source,
            ParseOptions::from(Mode::Module).with_target_version(PythonVersion::PY315),
        );
        assert_eq!(parsed.errors()[0].error.to_string(), message, "{source}");
    }
}

#[test]
fn corrected_locations() {
    let source = "match x:\n    case {**rest, 1: value}: pass";
    let parsed = parse_unchecked(source, Mode::Module.into());
    let error = &parsed.errors()[0];
    assert_eq!(
        error.error.to_string(),
        "double star pattern must be the last (right-most) subpattern in the mapping pattern"
    );
    assert_eq!(&source[error.location], "rest");

    for prefix in ["", "b", "f", "t"] {
        let source = format!("print(\n    {prefix}\"first\"\n    {prefix}\"last\"\n    x=1\n)");
        let parsed = parse_unchecked(
            &source,
            ParseOptions::from(Mode::Module).with_target_version(PythonVersion::PY315),
        );
        let error = &parsed.errors()[0];
        assert_eq!(
            error.error.to_string(),
            "invalid syntax. Perhaps you forgot a comma?"
        );
        assert_eq!(&source[error.location], format!("{prefix}\"last\""));
    }
}

#[test]
fn diagnostic_probes_preserve_valid_syntax() {
    for source in ["assert (x := 1)", "assert True, (message := 2)", "case(1)"] {
        let parsed = parse_unchecked(source, Mode::Module.into());
        assert!(
            parsed.errors().is_empty(),
            "{source}: {:?}",
            parsed.errors()
        );
    }
}
