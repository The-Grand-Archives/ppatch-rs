use std::{
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

#[derive(thiserror::Error, Debug)]
pub enum ParamdexFetchError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Command {cmd} failed with exit code {status}: \n{output}")]
    CommandFailed {
        cmd: String,
        status: ExitStatus,
        output: String,
    },
}

pub struct ParamdexFetchConfig {
    git_url: String,
    branch_or_tag: String,
    path_to_game: String,
}

impl ParamdexFetchConfig {
    /// Attempt to fetch a game-specific paramdex repository, cloning it to the provided path.
    pub fn fetch(&self, path: &Path) -> Result<PathBuf, ParamdexFetchError> {
        let clone_out = Command::new("git")
            .args(["clone", "-n", "--depth=1", "--filter=tree:0 --sparse -b"])
            .arg(&self.branch_or_tag)
            .arg(&self.git_url)
            .arg(path)
            .output()?;

        if !clone_out.status.success() {
            return Err(ParamdexFetchError::CommandFailed {
                cmd: "git clone".to_owned(),
                status: clone_out.status,
                output: String::from_utf8_lossy(&clone_out.stderr).to_string(),
            });
        }

        let checkout_out = Command::new("git")
            .current_dir(path)
            .args(["sparse-checkout", "set", "--no-cone"])
            .arg(self.path_to_game.to_owned() + "/Defs")
            .arg(self.path_to_game.to_owned() + "/Meta")
            .arg(self.path_to_game.to_owned() + "/Enums.json")
            .output()?;

        if !checkout_out.status.success() {
            return Err(ParamdexFetchError::CommandFailed {
                cmd: "git sparse-checkout".to_owned(),
                status: checkout_out.status,
                output: String::from_utf8_lossy(&checkout_out.stderr).to_string(),
            });
        }

        Ok(path.join(&self.path_to_game))
    }
}
