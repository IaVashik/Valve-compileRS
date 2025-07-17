use valve_compilers::{Compiler, CompilerArg};
use valve_compilers::vbsp::{Vbsp, VbspArg};

/// Test 1.1: Validate the static metadata of a specific compiler.
#[test]
fn test_vbsp_metadata() {
    let compiler = Vbsp::default();

    assert_eq!(compiler.name(), "VBSP");
    assert_eq!(compiler.description(), "BSP Compiler for Source Engine");
    assert_eq!(compiler.working_dir_template(), "$binDir");

    // Test the get_metadata() convenience method
    let metadata = compiler.get_metadata();
    assert_eq!(metadata.name, "VBSP");
}

/// Test 1.2: Check that the Default implementation populates args correctly.
#[test]
fn test_vbsp_default_arguments() {
    let compiler = Vbsp::default();
    let default_args = compiler.get_args();

    // Check that the default args list contains the expected arguments.
    assert!(!default_args.is_empty());
    // todo
}

/// Test 1.3: Validate the build_args() logic for a compiler instance.
#[test]
fn test_vbsp_build_args() {
    let mut compiler = Vbsp::new();

    // Test with an empty list
    assert_eq!(compiler.build_args().trim(), "");

    // Add arguments
    compiler.add_arg(VbspArg::NoDetail);
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(5));
    compiler.add_arg(VbspArg::Verbose);

    // Test with a populated list
    let expected = "-nodetail -staticpropcombine_mininstances 5 -verbose";
    let built_args = compiler.build_args();
    assert_eq!(built_args.trim(), expected);

    // Test clearing arguments
    compiler.clear_args();
    assert_eq!(compiler.build_args().trim(), "");
}

/// Test 1.4: Validate the top-level CompilerEnum correctly delegates calls.
#[test]
fn test_compiler_enum_delegation() {
    let enum_compiler = valve_compilers::CompilerEnum::Vbsp(Vbsp::default());
    assert_eq!(enum_compiler.name(), "VBSP");
    assert_eq!(enum_compiler.description(), "BSP Compiler for Source Engine");

    // Test the delegated build_args as well
    let mut vrad_instance = valve_compilers::vrad::Vrad::new();
    vrad_instance.add_arg(valve_compilers::vrad::VradArg::Fast);
    let vrad_enum = valve_compilers::CompilerEnum::Vrad(vrad_instance);

    assert_eq!(vrad_enum.build_args().trim(), "-fast");
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

    // Iterate over all possible Vbsp arguments and assert that they are compatible with CS:GO.
    for arg_variant in VbspArg::iter() {
        if arg_variant == VbspArg::AllowDynamicPropsAsStatic {continue;} // only in GMod
        assert!(
            arg_variant.is_compatible_with_game(current_game_id),
            "'{}' should be compatible with CS:GO!",
            arg_variant.name()
        );
    }
}
