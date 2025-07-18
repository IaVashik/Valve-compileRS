use std::path::PathBuf;

/// Defines the type of value an argument can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Flag,
    Float,
    Integer,
    String,
    Path,
}

/// Defines the common interface for a compiler tool.
pub trait Compiler: Default {
    /// The specific argument type associated with this compiler.
    type Arg: CompilerArg;

    /// Returns the human-readable name of the compiler.
    fn name(&self) -> &'static str;
    /// Returns a brief description of what the compiler does.
    fn description(&self) -> &'static str;
    /// Returns the default working directory for the compiler.
    fn working_dir_template(&self) -> &'static str;

    /// Returns a slice of the arguments configured for this compiler instance.
    fn get_args(&self) -> &[Self::Arg];
    /// Adds a configured argument to this compiler instance.
    fn add_arg(&mut self, arg: Self::Arg);
    /// Removes all user-configured arguments. Base arguments are unaffected.
    fn clear_args(&mut self);

    /// Convenience method for getting all metadata at once
    fn get_metadata(&self) -> CompilerMetadata {
        CompilerMetadata {
            name: self.name(),
            description: self.description(),
            working_dir_template: self.working_dir_template(),
        }
    }

    /// Returns the configured arguments in a structured format.
    fn get_structured_args(&self) -> Vec<(&'static str, Option<String>)> {
        self.get_args().iter().map(|arg| arg.as_arg()).collect()
    }

    /// Builds the final, flattened list of command-line arguments for execution.
    fn build_args(&self) -> Vec<String> {
        let structured_args = self.get_structured_args();
        let mut final_args = Vec::with_capacity(structured_args.len() * 2);

        for (key, value_opt) in structured_args {
            if !key.is_empty() {
                final_args.push(key.to_string());
            }

            if let Some(value) = value_opt {
                final_args.push(value);
            }
        }

        final_args
    }

    /// Build the final command-line string for execution using context values.///
    /// If provided, `executable` will be used as the compiler path. Otherwise,
    /// it defaults to `<bin_dir>/<compiler_name>.exe`.
    fn build_command(&self, context: &CompilerContext, executable: Option<PathBuf>) -> CommandInfo {
        let final_args = self
            .build_args()
            .iter()
            .map(|arg| context.replace(arg))
            .collect();
        let resolved_wd = PathBuf::from(context.replace(self.working_dir_template()));

        let compiler_path = if let Some(path) = executable {
            path
        } else {
            context.bin_dir.join(&self.name().to_lowercase()).with_extension("exe")
        };

        CommandInfo {
            name: self.name(),
            compiler_path: compiler_path,
            args: final_args,
            working_dir: resolved_wd,
        }
    }
}

/// Defines the common interface for a compiler argument.
pub trait CompilerArg: Sized {
    /// Returns the human-readable name of the argument.
    fn name(&self) -> &'static str;
    /// Returns a detailed description of the argument's purpose.
    fn description(&self) -> &'static str;
    /// Returns the type of value this argument holds (e.g., Flag, Float).
    fn value_type(&self) -> ValueType;
    /// Returns the default value for this argument, if one is defined.
    fn get_default_value(&self) -> Option<Self>;
    /// Formats the argument and its value (if any) into a command-line string.
    fn as_arg(&self) -> (&'static str, Option<String>);
    /// Whether this argument is used by the compiler by default.
    fn is_default(&self) -> bool;

    /// Returns a slice of game App IDs this argument is compatible with.
    /// Returns `None` if the argument is universally compatible (i.e., has no game constraints).
    fn compatible_games(&self) -> Option<&'static [u32]>;
    /// Checks if this argument is compatible with a specific game App ID.
    fn is_compatible_with_game(&self, app_id: u32) -> bool {
        match self.compatible_games() {
            Some(games) => games.contains(&app_id),
            None => true,
        }
    }
}

/// Holds the concrete values for placeholders used in compiler arguments.
#[derive(Debug, Clone, Default)]
pub struct CompilerContext {
    // Base paths
    pub bin_dir: PathBuf,
    pub game_dir: PathBuf,
    pub map_path: PathBuf,
    pub out_dir: PathBuf,

    // Derived values (computed at creation)
    pub map_dir: PathBuf,          // Directory containing the map
    pub map_name: String,          // Filename without extension (e.g., "de_dust2")
    pub map_name_ext: String,      // Filename with extension (e.g., "de_dust2.vmf")
    pub map_ext: String,           // File extension (e.g., "vmf")
    pub bsp_path: PathBuf,         // Full path to the .bsp file
}

impl CompilerContext {
    pub fn new(
        bin_dir: Option<PathBuf>,
        game_dir: Option<PathBuf>,
        map_path: Option<PathBuf>,
        out_dir: Option<PathBuf>,
    ) -> Self {
        let map_path = map_path.unwrap_or_default();
        let map_dir = map_path.parent().map(PathBuf::from).unwrap_or_default();

        // If out_dir is not specified, it defaults to the map directory
        let out_dir = out_dir.unwrap_or_else(|| map_dir.clone());

        // Calculate all derived values
        let map_name = map_path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let map_name_ext = map_path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let map_ext = map_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let bsp_path = map_path.with_extension("bsp");

        Self {
            bin_dir: bin_dir.unwrap_or_default(),
            game_dir: game_dir.unwrap_or_default(),
            map_path: map_path.clone(),
            out_dir,
            map_dir,
            map_name,
            map_name_ext,
            map_ext,
            bsp_path,
        }
    }

    /// Replaces placeholders in the string in a single pass and returns a new string.
    pub fn replace(&self, input: &str) -> String {
        // Pre-allocate memory to avoid reallocations.
        // Take the original length + a small buffer.
        let mut output = String::with_capacity(input.len() + 32);
        let mut last_match_end = 0;

        // Iterate over all occurrences of the '$' character
        for (start_of_match, _) in input.match_indices('$') {
            // Skip if this part of the string has already been processed
            if start_of_match < last_match_end {
                continue;
            }

            // Add the part of the string from the end of the last match to the start of the current one
            output.push_str(&input[last_match_end..start_of_match]);

            // Check what follows the '$'
            let remaining = &input[start_of_match..];
            let placeholder_value = self.get_placeholder_value(remaining);

            if let Some((placeholder_key, value)) = placeholder_value {
                // Placeholder found, append its value
                output.push_str(&value);
                // Advance the cursor by the length of the placeholder (e.g., "$mapName")
                last_match_end = start_of_match + placeholder_key.len() + 1; // +1 for '$'
            } else {
                // This is not a placeholder, just a '$' character. Add it as is.
                output.push('$');
                last_match_end = start_of_match + 1;
            }
        }

        // Add the remaining tail of the string after the last match
        if last_match_end < input.len() {
            output.push_str(&input[last_match_end..]);
        }

        output
    }

    /// Helper function for matching a string slice with known placeholders.
    /// Returns (key, value) on success.
    fn get_placeholder_value(&self, remaining_slice: &str) -> Option<(&str, String)> {
        // We use starts_with, which is a very fast operation for &str
        if remaining_slice.starts_with("$binDir") {
            Some(("binDir", self.bin_dir.to_string_lossy().into_owned()))
        } else if remaining_slice.starts_with("$gameDir") {
            Some(("gameDir", self.game_dir.to_string_lossy().into_owned()))
        } else if remaining_slice.starts_with("$mapPath") {
            Some(("mapPath", self.map_path.to_string_lossy().into_owned()))
        } else if remaining_slice.starts_with("$outDir") {
            Some(("outDir", self.out_dir.to_string_lossy().into_owned()))
        } else if remaining_slice.starts_with("$mapDir") {
            Some(("mapDir", self.map_dir.to_string_lossy().into_owned()))
        } else if remaining_slice.starts_with("$mapNameExt") {
            Some(("mapNameExt", self.map_name_ext.clone()))
        } else if remaining_slice.starts_with("$mapName") { // Important: $mapNameExt must come before $mapName
            Some(("mapName", self.map_name.clone()))
        } else if remaining_slice.starts_with("$mapExt") {
            Some(("mapExt", self.map_ext.clone()))
        } else if remaining_slice.starts_with("$bspPath") {
            Some(("bspPath", self.bsp_path.to_string_lossy().into_owned()))
        }
        // Aliases
        else if remaining_slice.starts_with("$file") {
            Some(("file", self.map_name.clone()))
        } else if remaining_slice.starts_with("$path") {
            Some(("path", self.map_path.to_string_lossy().into_owned()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: &'static str,
    /// The path to the compiler executable.
    pub compiler_path: PathBuf,
    /// The complete, resolved argument list.
    pub args: Vec<String>,
    /// The final working directory for the command.
    pub working_dir: PathBuf,
}

/// A struct for convenience, serialization, and data transfer.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize, serde::Deserialize))]
pub struct CompilerMetadata {
    /// The human-readable name of the compiler.
    pub name: &'static str,
    /// A brief description of what the compiler does.
    pub description: &'static str,
    /// The default working directory for the compiler.
    pub working_dir_template: &'static str,
}

#[derive(Debug, PartialEq)]
pub enum ParseArgError {
    /// The provided argument was not recognized by the compiler.
    UnknownArgument(String),
    /// An argument that requires a value was provided without one.
    MissingValue(&'static str),
    /// A flag argument, which does not accept a value, was provided with one.
    UnexpectedValue(&'static str),
    /// The value provided for an argument was not in the expected format.
    InvalidValue {
        argument: &'static str,
        value: String,
    },
}

impl std::fmt::Display for ParseArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownArgument(arg) => write!(f, "unknown argument: {}", arg),
            Self::MissingValue(arg) => write!(f, "argument '{}' requires a value, but none was provided", arg),
            Self::UnexpectedValue(arg) => write!(f, "argument '{}' is a flag and does not accept a value", arg),
            Self::InvalidValue { argument, value } => write!(f, "invalid value '{}' for argument '{}'", value, argument),
        }
    }
}

impl std::error::Error for ParseArgError {}

include!(concat!(env!("OUT_DIR"), "/generated_compilers.rs"));
