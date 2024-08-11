use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output},
};

use serde_derive::{Deserialize, Serialize};

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
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamdexGitFetch {
    git_url: String,
    branch: Option<String>,
    paramdex_path: String,
    games: Vec<String>,
}

trait ExecCmd {
    fn exec_command(&mut self) -> Result<Output, ParamdexFetchError>;
}
impl ExecCmd for Command {
    fn exec_command(&mut self) -> Result<Output, ParamdexFetchError> {
        let out = self.output()?;
        if !out.status.success() {
            return Err(ParamdexFetchError::CommandFailed {
                cmd: format!("{:?}", self),
                status: out.status,
                output: String::from_utf8_lossy(&out.stderr).to_string(),
            });
        }
        Ok(out)
    }
}

impl ParamdexGitFetch {
    pub fn new(git_url: impl AsRef<str>) -> Self {
        Self {
            git_url: git_url.as_ref().to_string(),
            branch: None,
            paramdex_path: ".".to_string(),
            games: Vec::new(),
        }
    }

    pub fn branch(&mut self, branch: impl AsRef<str>) -> &mut Self {
        self.branch = Some(branch.as_ref().to_string());
        self
    }

    pub fn paramdex_path(&mut self, path: impl AsRef<str>) -> &mut Self {
        self.paramdex_path = path.as_ref().to_string();
        self
    }

    pub fn games<S: AsRef<str>>(&mut self, path: impl IntoIterator<Item = S>) -> &mut Self {
        self.games.extend(path.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    /// Attempt to fetch a paramdex repository from a remote Git repo, cloning it to the provided path.
    /// Uses sparse checkouts to fetch only the files at a specific path for the given games.
    ///
    /// Returns the root paramdex path.
    pub fn fetch(&self, path: impl AsRef<Path>) -> Result<PathBuf, ParamdexFetchError> {
        let path = path.as_ref();

        Command::new("git")
            .args(["clone", "-n", "--depth=1", "--filter=tree:0", "--sparse"])
            .args(self.branch.as_ref().map(|b| vec!["-b", b]).unwrap_or(Default::default()))
            .arg(&self.git_url)
            .arg(path)
            .exec_command()?;

        Command::new("git")
            .current_dir(path)
            .args(["sparse-checkout", "set", "--no-cone"])
            .args(self.games.iter().flat_map(|g| {
                let p = &self.paramdex_path;
                [
                    format!("{p}/{g}/Defs"),
                    format!("{p}/{g}/Meta"),
                    format!("{p}/{g}/Enums.json"),
                ]
            }))
            .exec_command()?;

        Command::new("git").current_dir(path).arg("checkout").exec_command()?;

        Ok(std::fs::canonicalize(path.join(&self.paramdex_path))?)
    }

    /// Attempt to fetch a paramdex repository from a remote Git repo, cloning it to the provided path.
    /// Uses sparse checkouts to fetch only the files at a specific path for the given games.
    ///
    /// This is different from [`ParamdexGitFetch::fetch`] in two ways:
    /// - `path` is created if some of the folders comprising it don't exist;
    /// - The fetch operation is cached based on the fields of this [`ParamdexGitFetch`] object.
    ///     If the last fetch was made from the same source, it will not happen again.
    ///
    /// Returns the root paramdex path.
    pub fn fetch_cached(&self, path: impl AsRef<Path>) -> Result<PathBuf, ParamdexFetchError> {
        let path = path.as_ref();
        const META_FILE_NAME: &'static str = ".paramdex_fetch_meta.json";

        let should_fetch = match std::fs::read(path.join(META_FILE_NAME)) {
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Ok(true)
                } else {
                    Err(e)
                }
            }
            Ok(contents) => Ok(serde_json::de::from_slice::<ParamdexGitFetch>(&contents)? != *self),
        }?;

        if !should_fetch {
            return Ok(std::fs::canonicalize(path.join(&self.paramdex_path))?);
        }

        std::fs::remove_dir_all(path).ok();
        std::fs::create_dir_all(path)?;

        let out_path = self.fetch(path)?;
        std::fs::write(path.join(META_FILE_NAME), serde_json::to_vec_pretty(self)?)?;

        Ok(out_path)
    }
}
