use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use enums::{ProjectEnum, ProjectEnums};
use meta::ParamMeta;
use paramdef::Paramdef;

pub mod enums;
pub mod git_fetch;
pub mod meta;
pub mod paramdef;

pub struct DefWithMeta {
    pub def: Paramdef,
    pub meta: Option<ParamMeta>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParamdexLoadError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("XML error: {0}")]
    XmlError(#[from] quick_xml::DeError),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub struct Paramdex {
    path: PathBuf,
    enums: HashMap<String, ProjectEnum>,
    ext_defs: HashMap<String, DefWithMeta>,
}

impl Paramdex {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            enums: Default::default(),
            ext_defs: Default::default(),
        }
    }

    pub fn load_defs(&mut self) -> Result<&mut Self, ParamdexLoadError> {
        let defs_path = self.path.join("Defs");
        for entry in std::fs::read_dir(defs_path)? {
            let fpath = entry?.path();

            if fpath.extension() != Some(OsStr::new("xml")) {
                continue;
            };
            let def_name = match fpath.file_stem() {
                None => continue,
                Some(n) => n.to_string_lossy().to_string(),
            };
            let def_contents = std::fs::read_to_string(fpath)?;
            self.ext_defs.insert(
                def_name,
                DefWithMeta {
                    def: quick_xml::de::from_str(&def_contents)?,
                    meta: None,
                },
            );
        }
        Ok(self)
    }

    pub fn load_metas(&mut self) -> Result<&mut Self, ParamdexLoadError> {
        let metas_path = self.path.join("Meta");
        for entry in std::fs::read_dir(metas_path)? {
            let fpath = entry?.path();

            if fpath.extension() != Some(OsStr::new("xml")) {
                continue;
            };
            let def_name = match fpath.file_stem() {
                None => continue,
                Some(n) => n.to_string_lossy(),
            };
            if let Some(pair) = self.ext_defs.get_mut(def_name.as_ref()) {
                let meta_contents = std::fs::read_to_string(fpath)?;
                pair.meta = Some(quick_xml::de::from_str(&meta_contents)?);
            }
        }
        Ok(self)
    }

    pub fn load_enums(&mut self) -> Result<&mut Self, ParamdexLoadError> {
        let enums_content = std::fs::read(self.path.join("Enums.json"))?;
        let enums: ProjectEnums = serde_json::from_slice(&enums_content)?;
        self.enums = enums.list.into_iter().map(|e| (e.name.clone(), e)).collect();
        Ok(self)
    }

    pub fn compute_def_layouts(&mut self, version: u64) -> &mut Self {
        for def in self.ext_defs.values_mut().map(|pair| &mut pair.def) {
            def.compute_field_offsets(version);
        }
        self
    }

    pub fn defs(&self) -> impl Iterator<Item = &Paramdef> {
        self.ext_defs.values().map(|pair| &pair.def)
    }
}
