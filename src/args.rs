use clap:: {
    Args,
    Parser,
    Subcommand
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct TotalArgs {
    #[clap(subcommand)]
    pub entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    //Create Programs
    Create(CreateProgram),
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
pub struct RunProgram {
    //The path or name of the project to run
    pub path: String,
}


