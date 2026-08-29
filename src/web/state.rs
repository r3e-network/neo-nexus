//! Server state shared by every web handler: the workspace repository, the
//! data directory (workspace children such as managed configs and logs live
//! beside the database), and the authentication store.

use std::path::PathBuf;

use crate::repository::Repository;

use super::auth::AuthStore;

#[derive(Clone)]
pub struct WebState {
    pub repository: Repository,
    pub data_dir: PathBuf,
    pub auth: AuthStore,
}

impl WebState {
    pub fn new(repository: Repository, data_dir: PathBuf, auth: AuthStore) -> Self {
        Self {
            repository,
            data_dir,
            auth,
        }
    }

    /// A subdirectory beside the database, mirroring the GUI and CLI
    /// conventions: managed configs under `nodes/`, supervised logs under
    /// `logs/`.
    pub fn workspace_child_dir(&self, child: &str) -> PathBuf {
        self.data_dir.join(child)
    }
}
