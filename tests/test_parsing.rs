use valve_compilers::ParseArgError;
use valve_compilers::vbsp::VbspArg;
use std::convert::TryFrom;
use std::path::PathBuf;

/// Test 2.1: Test successful parsing of all argument types.
#[test]
fn test_parsing_success_cases() {
    // Flag
    assert!(matches!(VbspArg::try_from("-onlyents").unwrap(), VbspArg::OnlyEntities));
    // Float
    let arg_float = VbspArg::try_from("-micro 0.5").unwrap();
    assert!(matches!(arg_float, VbspArg::MicroVolumeTest(v) if (v - 0.5).abs() < f32::EPSILON));
    // Integer
    let arg_int = VbspArg::try_from("-staticpropcombine_mininstances 42").unwrap();
    assert!(matches!(arg_int, VbspArg::StaticPropCombineMinInstances(42)));
    // Path (from a different compiler for variety)
    let bspzip_arg = valve_compilers::bspzip::BspzipArg::try_from("-addlist /my/path/list.txt").unwrap();
    assert!(matches!(bspzip_arg, valve_compilers::bspzip::BspzipArg::PackFileList(p) if p == PathBuf::from("/my/path/list.txt")));
}

/// Test 2.2: Test all defined parsing error conditions.
#[test]
fn test_parsing_error_cases() {
    // Error: Unknown argument
    let err1 = VbspArg::try_from("-nonexistent_arg").unwrap_err();
    assert_eq!(err1, ParseArgError::UnknownArgument("-nonexistent_arg".to_string()));

    // Error: Missing value for an argument that requires one
    let err2 = VbspArg::try_from("-micro").unwrap_err();
    assert_eq!(err2, ParseArgError::MissingValue("-micro"));

    // Error: Invalid value type (e.g., text for a number)
    let err3 = VbspArg::try_from("-micro not_a_float").unwrap_err();
    assert_eq!(err3, ParseArgError::InvalidValue { argument: "-micro", value: "not_a_float".to_string() });

    // Error: Extraneous value provided for a flag
    let err4 = VbspArg::try_from("-verbose some_value").unwrap_err();
    assert_eq!(err4, ParseArgError::UnexpectedValue("-verbose"));
}

/// Test 2.3: Test parsing from an owned String.
#[test]
fn test_parsing_from_owned_string() {
    let owned_string = String::from("-nodetail");
    let arg = VbspArg::try_from(owned_string).unwrap();
    assert!(matches!(arg, VbspArg::NoDetail));
}
