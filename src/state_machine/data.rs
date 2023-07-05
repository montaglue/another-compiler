use crate::internal_representations::gast::Program;

pub struct CompilerData {
    pub(super) config: Config,
    pub(super) code: Option<String>,
    pub(super) ast: Option<Program>,
}

impl CompilerData {
    pub fn new(config: Config) -> Self {
        CompilerData {
            config,
            code: None,
            ast: None,
        }
    }
}

pub struct Config {
    pub project_directory: String,
}

impl Config {
    pub fn from_args(_args: Vec<String>) -> anyhow::Result<Self> {
        let project_directory = std::env::current_dir()?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Could not convert path to string"))?
            .to_string();

        println!("Project directory: {}", project_directory);
        Ok(Config { project_directory })
    }
}
