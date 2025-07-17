use valve_compilers::{Compiler, CompilerContext, CommandInfo};
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
    // The MapFile argument is a special case as it has no argument flag, just the value


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

    // Expected arguments, ensuring order and correct formatting for different types.
    // Note: The default values should be applied first, then user-added arguments.
    // -game and map_path are default arguments, but are overridden by later explicit additions.
    let expected_args = [
        "-game", r"C:\CustomGame\game", // Overridden default
        r"C:\maps\my_map_final.vmf", // Overridden default
        "-verbose",
        "-onlyents",
        "-micro 0.5",
        "-staticpropcombine_mininstances 10",
    ].join(" ");
    assert_eq!(command_info.args.trim(), expected_args);

    let expected_wd = PathBuf::from(r"C:\Steam\steamapps\common\Counter-Strike Global Offensive\bin");
    assert_eq!(command_info.working_dir, expected_wd);
}

/// Test 3.2: Test context replacement with empty context values and default arguments.
#[test]
fn test_context_replacement_with_empty_context() {
    let compiler = Vbsp::default(); // Uses default args: -game $gameDir and $mapPath
    let context = CompilerContext::default(); // All paths are empty

    let executable = PathBuf::from("vbsp");

    let command_info = compiler.build_command(&context, Some(executable.clone()));

    // With an empty context, $binDir, $gameDir, and $mapPath placeholders should be replaced with nothing.
    assert_eq!(command_info.working_dir, PathBuf::from(""));
    assert_eq!(command_info.compiler_path, executable);

    // Default arguments for VBSP are "-game $gameDir" and "$mapPath".
    // When context is empty, they should resolve to "-game " and "".
    let expected_args = "-game  "; // Note the double space because empty gameDir.
    assert_eq!(command_info.args.trim(), expected_args.trim());
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
    let expected_args = format!("-game {} {}", context.game_dir.display(), context.map_path.display());
    assert_eq!(command_info.args.trim(), expected_args);
    assert_eq!(command_info.working_dir, context.bin_dir);
}

/// Test 3.4: Verifies that arguments with default values are handled correctly.
#[test]
fn test_default_value_arguments() {
    let mut compiler = Vbsp::default();
    // Add an argument with a default value, but don't specify the value
    compiler.add_arg(VbspArg::MicroVolumeTest(1.0)); // Default value from vbsp.toml is "1.0"
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(3)); // Default value from vbsp.toml is "3"

    let context = CompilerContext {
        bin_dir: PathBuf::from("/bin"),
        game_dir: PathBuf::from("/game"),
        map_path: PathBuf::from("/map.vmf"),
        ..Default::default()
    };

    let command_info = compiler.build_command(&context, None);

    let expected_args = format!(
        "-game {} {} -micro 1 -staticpropcombine_mininstances 3",
        context.game_dir.display(),
        context.map_path.display()
    );
    assert_eq!(command_info.args.trim(), expected_args);
}

/// Test 3.5: Verifies that flag arguments are correctly added and formatted.
#[test]
fn test_flag_arguments() {
    let mut compiler = Vbsp::default();
    compiler.add_arg(VbspArg::OnlyEntities);
    compiler.add_arg(VbspArg::NoWater);
    compiler.add_arg(VbspArg::Verbose);

    let context = CompilerContext::default(); // Context doesn't affect flags directly.
    let executable_path = PathBuf::from("vbsp.exe");
    let command_info = compiler.build_command(&context, Some(executable_path.clone()));

    let expected_args = "-game   -onlyents -nowater -verbose"; // With empty context for defaults
    assert_eq!(command_info.args.trim(), expected_args.trim());
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

    let expected_args = format!(
        "-game {} {}",
        r"C:\Program Files (x86)\Steam\steamapps\common\MyGame",
        r"D:\Maps For Game\level_01.vmf"
    );
    assert_eq!(command_info.args.trim(), expected_args);
}

/// Test 3.7: Verifies that negative float and integer arguments are handled.
#[test]
fn test_negative_number_arguments() {
    let mut compiler = Vbsp::default();
    compiler.add_arg(VbspArg::MicroVolumeTest(-0.1));
    // No explicit negative integer parameter in vbsp.toml, using min_instances for demonstration
    compiler.add_arg(VbspArg::StaticPropCombineMinInstances(-5));

    let context = CompilerContext::default();
    let executable_path = PathBuf::from("vbsp.exe");
    let command_info = compiler.build_command(&context, Some(executable_path.clone()));

    let expected_args = "-game   -micro -0.1 -staticpropcombine_mininstances -5";
    assert_eq!(command_info.args.trim(), expected_args.trim());
}
