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
    },
    /// Import saved profiles from an export bundle
    Import {
        /// Read the export bundle from this file (fails on id/label conflicts)
        #[arg(long, value_name = "file")]
        input: PathBuf,
    },
    /// Run local diagnostics
    Doctor {
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
        #[arg(long)]
        all: bool,
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
        "Examples:\n  {name} save --label work\n  {name} load --label work\n  {name} load --id mail@example.com-team --force\n  {name} list\n  {name} list --json\n  {name} export --output profiles-export.json\n  {name} import --input profiles-export.json\n  {name} doctor\n  {name} doctor --json\n  {name} label set --id mail@example.com-team --to work\n  {name} label clear --label work\n  {name} status\n  {name} status --json\n  {name} status --all --json\n  {name} delete --label work\n  {name} delete --id mail@example.com-team --yes"
    )
}
