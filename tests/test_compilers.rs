use valve_compilers::{Compiler, CompilerArg};
use valve_compilers::vbsp::{Vbsp, VbspArg};

/// Test 1.1: Validate the static metadata of a specific compiler.
#[test]
fn test_vbsp_metadata() {
    let compiler = Vbsp::default();

    assert_eq!(compiler.name(), "VBSP");
    assert_eq!(compiler.description(), "BSP Compiler for Source Engine");
    assert_eq!(compiler.working_dir_template(), "$binDir");

    let metadata = compiler.get_metadata();
    assert_eq!(metadata.name, "VBSP");
}

/// Test 1.2: Check that the Default implementation populates args correctly.
#[test]
fn test_vbsp_default_arguments() {
    let compiler = Vbsp::default();
    let default_args = compiler.get_args();

    assert_eq!(default_args.len(), 2);
    assert!(matches!(default_args[0], VbspArg::GameDirectory(_)));
    assert!(matches!(default_args[1], VbspArg::MapFile(_)));
}

/// Test 1.3: Validate the build_args() logic for a compiler instance.
#[test]
fn test_vbsp_build_args() {
    let mut compiler = Vbsp::new();

    assert!(compiler.build_args().is_empty());

    compiler.add_arg(VbspArg::NoDetail);
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(5));
    compiler.add_arg(VbspArg::Verbose);

    let expected: Vec<String> = vec![
        "-nodetail",
        "-staticpropcombine_mininstances",
        "5",
        "-verbose",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let built_args = compiler.build_args();
    assert_eq!(built_args, expected);

    compiler.clear_args();
    assert!(compiler.build_args().is_empty());
}

/// Test 1.4: Validate the top-level CompilerEnum correctly delegates calls.
#[test]
fn test_compiler_enum_delegation() {
    let enum_compiler = valve_compilers::CompilerEnum::Vbsp(Vbsp::default());
    assert_eq!(enum_compiler.name(), "VBSP");
    assert_eq!(enum_compiler.description(), "BSP Compiler for Source Engine");

    let mut vrad_instance = valve_compilers::vrad::Vrad::new();
    vrad_instance.add_arg(valve_compilers::vrad::VradArg::Fast);
    let vrad_enum = valve_compilers::CompilerEnum::Vrad(vrad_instance);

    let expected: Vec<String> = vec!["-fast".to_string()];
    assert_eq!(vrad_enum.build_args(), expected);
}

/// Test 1.5 (Conditional): Test serialization feature.
#[test]
#[cfg(feature = "serialization")]
fn test_serialization_round_trip() {
    let mut compiler = Vbsp::default();
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(10));

    let json_string = serde_json::to_string(&compiler).expect("Failed to serialize");
    let deserialized: Vbsp = serde_json::from_str(&json_string).expect("Failed to deserialize");

    assert_eq!(compiler.build_args(), deserialized.build_args());
}

/// Test 1.6 (Conditional): Test game compatibility.
#[test]
#[cfg(feature = "enum_iter")]
fn test_csgo_compatibility() {
    use strum::IntoEnumIterator;
    let current_game_id = 730; // CS:GO

    for arg_variant in VbspArg::iter() {
        if arg_variant == VbspArg::AllowDynamicPropsAsStatic {continue;} // only in GMod
        assert!(
            arg_variant.is_compatible_with_game(current_game_id),
            "'{}' should be compatible with CS:GO!",
            arg_variant.name()
        );
    }
}
