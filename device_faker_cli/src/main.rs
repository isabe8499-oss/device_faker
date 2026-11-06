//! device_faker_cli — developer/diagnostic command line tool.
//!
//! The CLI is part of the Cargo development workspace, but it is not copied
//! into the Magisk/KernelSU/APatch module ZIP.

use anyhow::Result;
use argh::FromArgs;

mod converter;
mod identity;

/// Device Faker configuration tool
#[derive(FromArgs)]
struct Cli {
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    /// Convert Magisk module ZIP to TOML configuration
    Convert(ConvertArgs),
    /// Convert a system.prop-style file to TOML configuration
    ConvertProps(ConvertPropsArgs),
    /// Read current device properties via getprop and export TOML configuration
    DumpDevice(DumpDeviceArgs),
    /// Generate synthetic, app-scoped identity test profiles
    Identity(IdentityArgs),
}

/// Convert Magisk module ZIP to TOML configuration
#[derive(FromArgs)]
#[argh(subcommand, name = "convert")]
struct ConvertArgs {
    /// input ZIP file path
    #[argh(option, short = 'i', long = "input")]
    input: String,

    /// output file path
    #[argh(option, short = 'o', long = "output")]
    output: String,
}

/// Convert a system.prop-style file to TOML configuration
#[derive(FromArgs)]
#[argh(subcommand, name = "convert-props")]
struct ConvertPropsArgs {
    /// input system.prop or property text file path
    #[argh(option, short = 'i', long = "input")]
    input: String,

    /// output file path
    #[argh(option, short = 'o', long = "output")]
    output: String,
}

/// Read current device properties via getprop and export TOML configuration
#[derive(FromArgs)]
#[argh(subcommand, name = "dump-device")]
struct DumpDeviceArgs {
    /// output TOML file path
    #[argh(option, short = 'o', long = "output")]
    output: String,
}

/// Identity commands
#[derive(FromArgs)]
#[argh(subcommand, name = "identity")]
struct IdentityArgs {
    #[argh(subcommand)]
    command: IdentityCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum IdentityCommand {
    Generate(IdentityGenerateArgs),
}

/// Generate a synthetic app-scoped test identity and print TOML or JSON.
#[derive(FromArgs)]
#[argh(subcommand, name = "generate")]
struct IdentityGenerateArgs {
    /// base Android package name without the @userId suffix
    #[argh(option, short = 'p', long = "package")]
    package: String,

    /// the Android user ID (default: 0)
    #[argh(option, short = 'u', long = "user-id")]
    user_id: Option<u32>,

    /// optional 64-character hexadecimal seed (32 bytes) for reproducible output
    #[argh(option, long = "seed")]
    seed: Option<String>,

    /// emit JSON instead of a TOML snippet
    #[argh(switch, long = "json")]
    json: bool,
}

fn main() -> Result<()> {
    let cli: Cli = argh::from_env();

    match cli.command {
        Command::Convert(args) => converter::convert_zip_config(&args.input, &args.output)?,
        Command::ConvertProps(args) => converter::convert_props_config(&args.input, &args.output)?,
        Command::DumpDevice(args) => converter::dump_current_device_config(&args.output)?,
        Command::Identity(args) => match args.command {
            IdentityCommand::Generate(args) => identity::generate(
                &args.package,
                args.user_id.unwrap_or(0),
                args.seed.as_deref(),
                args.json,
            )?,
        },
    }

    Ok(())
}
