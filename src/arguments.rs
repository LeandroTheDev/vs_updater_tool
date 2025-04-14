use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Items {
    #[arg(long, value_delimiter = ',')]
    pub ignore_folders: Option<Vec<String>>,

    #[arg(long, value_delimiter = ',')]
    pub ignore_files: Option<Vec<String>>,

    #[arg(long)]
    pub working_path: Option<String>,

    #[arg(long)]
    pub game_type: Option<String>,
}
