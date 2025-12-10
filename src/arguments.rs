use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Items {
    #[arg(long, value_delimiter = ',')]
    pub ignore_folders: Option<Vec<String>>,

    #[arg(long, value_delimiter = ',')]
    pub ignore_files: Option<Vec<String>>,

    #[arg(long, value_delimiter = ',')]
    pub generate_modpack: Option<Vec<String>>,

    #[arg(long)]
    pub working_path: Option<String>,

    #[arg(long)]
    pub game_type: Option<String>,

    #[arg(long)]
    pub ignore_game_update: bool,

    #[arg(long)]
    pub ignore_mod_update: bool,

    #[arg(long)]
    pub mods_path: Option<String>,

    #[arg(long)]
    pub force_url: Option<String>,

    #[arg(long)]
    pub no_pre_mods: bool,

    #[arg(long)]
    pub no_pre: bool,
}
