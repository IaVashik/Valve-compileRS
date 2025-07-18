use valve_compilers::{Compiler, CompilerContext};
use valve_compilers::vbsp::{Vbsp, VbspArg};
use std::path::PathBuf;

/// Test 3.1: Verifies that variable substitution works for arguments and working directory.
/// This test now includes various argument types and ensures correct argument formatting
/// and context replacement.
#[test]
fn test_contextual_command_building() {
    // 1. Create a compiler instance and add various arguments.
    let mut compiler = Vbsp::new();
    compiler.add_arg(VbspArg::GameDirectory(PathBuf::from(r"C:\CustomGame\game")));
    compiler.add_arg(VbspArg::MapFile(PathBuf::from(r"C:\maps\my_map_final.vmf")));
    compiler.add_arg(VbspArg::Verbose);
    compiler.add_arg(VbspArg::OnlyEntities);
    compiler.add_arg(VbspArg::MicroVolumeTest(0.5));
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(10));


    // 2. Define the runtime context.
    let context = CompilerContext {
        bin_dir: PathBuf::from(r"C:\Steam\steamapps\common\Counter-Strike Global Offensive\bin"),
        game_dir: PathBuf::from(r"C:\Steam\steamapps\common\Counter-Strike Global Offensive\csgo"),
        map_path: PathBuf::from(r""),
        ..Default::default()
    };

    // 3. Define the path to the executable.
    let executable_path = context.bin_dir.join("vbsp.exe");

    // 4. Build the final command.
    let command_info = compiler.build_command(&context, Some(executable_path.clone()));

    // 5. Assert all parts of the final CommandInfo are correct.
    assert_eq!(command_info.compiler_path, executable_path);

    let expected_args: Vec<String> = vec![
        "-game", r"C:\CustomGame\game",
        r"C:\maps\my_map_final.vmf",
        "-verbose",
        "-onlyents",
        "-micro", "0.5",
        "-staticpropcombine_mininstances", "10",
    ].into_iter().map(String::from).collect();
    assert_eq!(command_info.args, expected_args);

    let expected_wd = PathBuf::from(r"C:\Steam\steamapps\common\Counter-Strike Global Offensive\bin");
    assert_eq!(command_info.working_dir, expected_wd);
}

/// Test 3.2: Test context replacement with empty context values and default arguments.
#[test]
fn test_context_replacement_with_empty_context() {
    let compiler = Vbsp::default();
    let context = CompilerContext::default();

    let executable = PathBuf::from("vbsp");

    let command_info = compiler.build_command(&context, Some(executable.clone()));

    assert_eq!(command_info.working_dir, PathBuf::from(""));
    assert_eq!(command_info.compiler_path, executable);

    let expected_args: Vec<String> = vec!["-game".to_string(), "".to_string(), "".to_string()];
    assert_eq!(command_info.args, expected_args);
}

/// Test 3.3: Verifies that default arguments are correctly applied when no explicit arguments are added.
#[test]
fn test_default_arguments_only() {
    let compiler = Vbsp::default();

    let context = CompilerContext {
        bin_dir: PathBuf::from("/some/bin"),
        game_dir: PathBuf::from("/some/game"),
        map_path: PathBuf::from("/some/map.vmf"),
        ..Default::default()
    };

    let executable_path = context.bin_dir.join("vbsp.exe");
    let command_info = compiler.build_command(&context, Some(executable_path.clone()));

    assert_eq!(command_info.compiler_path, executable_path);
    let expected_args: Vec<String> = vec![
        "-game".to_string(),
        context.game_dir.display().to_string(),
        context.map_path.display().to_string(),
    ];
    assert_eq!(command_info.args, expected_args);
    assert_eq!(command_info.working_dir, context.bin_dir);
}

/// Test 3.4: Verifies that arguments with default values are handled correctly.
#[test]
fn test_default_value_arguments() {
    let mut compiler = Vbsp::default();
    compiler.add_arg(VbspArg::MicroVolumeTest(1.0));
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(3));

    let context = CompilerContext {
        bin_dir: PathBuf::from("/bin"),
        game_dir: PathBuf::from("/game"),
        map_path: PathBuf::from("/map.vmf"),
        ..Default::default()
    };

    let command_info = compiler.build_command(&context, None);

    let expected_args: Vec<String> = vec![
        "-game",
        &context.game_dir.display().to_string(),
        &context.map_path.display().to_string(),
        "-micro",
        "1",
        "-staticpropcombine_mininstances",
        "3",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(command_info.args, expected_args);
}

/// Test 3.5: Verifies that flag arguments are correctly added and formatted.
#[test]
fn test_flag_arguments() {
    let mut compiler = Vbsp::default();
    compiler.add_arg(VbspArg::OnlyEntities);
    compiler.add_arg(VbspArg::NoWater);
    compiler.add_arg(VbspArg::Verbose);

    let context = CompilerContext::default();
    let executable_path = PathBuf::from("vbsp.exe");
    let command_info = compiler.build_command(&context, Some(executable_path.clone()));

    let expected_args: Vec<String> =
        vec!["-game", "", "", "-onlyents", "-nowater", "-verbose"]
            .into_iter()
            .map(String::from)
            .collect();
    assert_eq!(command_info.args, expected_args);
}

/// Test 3.6: Verifies that path arguments are correctly formatted with spaces.
#[test]
fn test_path_arguments() {
    let mut compiler = Vbsp::new();
    compiler.add_arg(VbspArg::GameDirectory(PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\MyGame")));
    compiler.add_arg(VbspArg::MapFile(PathBuf::from(r"D:\Maps For Game\level_01.vmf")));

    let context = CompilerContext::default();
    let executable_path = PathBuf::from("vbsp.exe");
    let command_info = compiler.build_command(&context, Some(executable_path.clone()));

    let expected_args: Vec<String> = vec![
        "-game",
        r"C:\Program Files (x86)\Steam\steamapps\common\MyGame",
        r"D:\Maps For Game\level_01.vmf",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(command_info.args, expected_args);
}

/// Test 3.7: Verifies that negative float and integer arguments are handled.
#[test]
fn test_negative_number_arguments() {
    let mut compiler = Vbsp::default();
    compiler.add_arg(VbspArg::MicroVolumeTest(-0.1));
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(-5));

    let context = CompilerContext::default();
    let executable_path = PathBuf::from("vbsp.exe");
    let command_info = compiler.build_command(&context, Some(executable_path.clone()));

    let expected_args: Vec<String> = vec![
        "-game",
        "",
        "",
        "-micro",
        "-0.1",
        "-staticpropcombine_mininstances",
        "-5",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(command_info.args, expected_args);
}
