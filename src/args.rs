use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct TotalArgs {
    #[clap(subcommand)]
    pub entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    /// Initialize Total support in the current application
    Init,
    /// Remove files created by Total initialization
    Detach,
    //Create Programs
    Create(CreateProgram),
    #[clap(alias = "d", alias = "--d", alias = "--delete")]
    Delete(DeleteProgram),
    Run(RunProgram),
}

#[derive(Debug, Args)]
pub struct CreateProgram {
    //The language of the project
    pub language: String,
    //The title of the project
    pub title: String,
}
#[derive(Debug, Args)]
pub struct DeleteProgram {
    //The path of the program
    pub path: String,
}

#[derive(Debug, Args)]
#[clap(trailing_var_arg = true)]
pub struct RunProgram {
    // Optional when the current project contains .total/app.toml
    #[clap(allow_hyphen_values = true)]
    pub language: Option<String>,
    #[clap(num_args = 0.., allow_hyphen_values = true)]
    pub extra_args: Vec<String>,
}
