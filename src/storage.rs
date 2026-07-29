use std::{fs, io, path::Path};

use ron::ser::PrettyConfig;
use thiserror::Error;

use crate::model::Project;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("could not read project: {0}")]
    Read(#[source] io::Error),
    #[error("could not parse RON project: {0}")]
    Parse(#[source] ron::error::SpannedError),
    #[error("could not serialize project: {0}")]
    Serialize(#[source] ron::Error),
    #[error("could not write project: {0}")]
    Write(#[source] io::Error),
}

pub fn load_project(path: &Path) -> Result<Project, StorageError> {
    let source = fs::read_to_string(path).map_err(StorageError::Read)?;
    let mut project: Project = ron::from_str(&source).map_err(StorageError::Parse)?;
    project.validate();
    Ok(project)
}

pub fn save_project(path: &Path, project: &Project) -> Result<(), StorageError> {
    let pretty = PrettyConfig::new()
        .depth_limit(16)
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let source = ron::ser::to_string_pretty(project, pretty).map_err(StorageError::Serialize)?;
    let temporary = path.with_extension("ron.tmp");
    fs::write(&temporary, source).map_err(StorageError::Write)?;
    fs::rename(&temporary, path).map_err(StorageError::Write)
}

pub fn save_text(path: &Path, contents: &str) -> Result<(), StorageError> {
    fs::write(path, contents).map_err(StorageError::Write)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn project_round_trips_through_ron() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ratatelier-{nonce}.ron"));
        let project = Project::new("round trip", 9, 5);
        save_project(&path, &project).unwrap();
        let loaded = load_project(&path).unwrap();
        let _ = fs::remove_file(path);
        assert_eq!(project, loaded);
    }
}
