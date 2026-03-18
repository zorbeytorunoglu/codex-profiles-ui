use clap::{ArgAction, Command, CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

use crate::command_name;

#[derive(Parser)]
#[command(author, version, about, color = clap::ColorChoice::Never)]
pub struct Cli {
    /// Disable styling and separators
    #[arg(long, global = true)]
    pub plain: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Save the current auth.json as a profile
    Save {
        /// Optional label for the profile (must be unique)
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// Load a saved profile
    Load {
        /// Load the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long, conflicts_with = "id")]
        label: Option<String>,
        /// Load the profile matching this exact id
        #[arg(value_name = "profile-id")]
        #[arg(long, conflicts_with = "label")]
        id: Option<String>,
        /// Continue without saving the current unsaved profile first
        #[arg(long)]
        force: bool,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// List saved profiles
    List {
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
        /// Show profile ids in human-readable output
        #[arg(long, conflicts_with = "json")]
        show_id: bool,
    },
    /// Export saved profiles for backup or transfer
    Export {
        /// Export only the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long, conflicts_with = "id")]
        label: Option<String>,
        /// Export only the profile(s) matching these exact ids
        #[arg(value_name = "profile-id")]
        #[arg(long, conflicts_with = "label", action = ArgAction::Append)]
        id: Vec<String>,
        /// Write the export bundle to this new file
        #[arg(long, value_name = "file")]
        output: PathBuf,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// Import saved profiles from an export bundle
    Import {
        /// Read the export bundle from this file (fails on id or label conflicts)
        #[arg(long, value_name = "file")]
        input: PathBuf,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// Run local diagnostics
    Doctor {
        /// Apply safe repairs for profile storage metadata
        #[arg(long)]
        fix: bool,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// Manage saved profile labels
    Label {
        #[command(subcommand)]
        command: LabelCommands,
    },
    /// Show usage details for the current or saved profiles
    Status {
        /// Show usage for all saved profiles
        #[arg(long, conflicts_with = "label", conflicts_with = "id")]
        all: bool,
        /// Show usage for the saved profile matching this label
        #[arg(
            long,
            value_name = "label",
            conflicts_with = "id",
            conflicts_with = "all"
        )]
        label: Option<String>,
        /// Show usage for the saved profile matching this exact id
        #[arg(
            long,
            value_name = "profile-id",
            conflicts_with = "label",
            conflicts_with = "all"
        )]
        id: Option<String>,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
        /// Include errored profiles in --all output
        #[arg(long, requires = "all")]
        show_errors: bool,
    },
    /// Delete saved profiles
    Delete {
        /// Skip delete confirmation
        #[arg(long)]
        yes: bool,
        /// Delete the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long, conflicts_with = "id")]
        label: Option<String>,
        /// Delete the profile matching this exact id
        #[arg(value_name = "profile-id")]
        #[arg(long, conflicts_with = "label", action = ArgAction::Append)]
        id: Vec<String>,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum LabelCommands {
    /// Set or replace a label on a saved profile
    Set {
        /// Select the profile matching this existing label
        #[arg(value_name = "label")]
        #[arg(long, conflicts_with = "id", required_unless_present = "id")]
        label: Option<String>,
        /// Select the profile matching this exact id
        #[arg(value_name = "profile-id")]
        #[arg(long, conflicts_with = "label", required_unless_present = "label")]
        id: Option<String>,
        /// New label to assign
        #[arg(long, value_name = "label")]
        to: String,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// Clear the label on a saved profile
    Clear {
        /// Select the profile matching this existing label
        #[arg(value_name = "label")]
        #[arg(long, conflicts_with = "id", required_unless_present = "id")]
        label: Option<String>,
        /// Select the profile matching this exact id
        #[arg(value_name = "profile-id")]
        #[arg(long, conflicts_with = "label", required_unless_present = "label")]
        id: Option<String>,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
    /// Rename an existing label
    Rename {
        /// Existing label to rename
        #[arg(value_name = "label")]
        #[arg(long)]
        label: String,
        /// New label to assign
        #[arg(long, value_name = "label")]
        to: String,
        /// Print machine-readable JSON output
        #[arg(long)]
        json: bool,
    },
}

pub fn command_with_examples() -> Command {
    let name = command_name();
    let mut cmd = Cli::command();
    cmd.set_bin_name(name);
    cmd = cmd.after_help(examples_root(name));
    cmd
}

fn examples_root(name: &str) -> String {
    format!(
        "Common options:\n  --json  Print machine-readable JSON output for commands that support it\n\nExamples:\n  {name} save --label work\n  {name} load --label work\n  {name} list --json\n  {name} status --all --json\n  {name} export --output profiles-export.json\n  {name} import --input profiles-export.json\n  {name} delete --label work --yes\n\nRun `{name} help <command>` for command-specific options."
    )
}
