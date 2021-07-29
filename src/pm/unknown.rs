use async_trait::async_trait;

use super::Pm;
use crate::{dispatch::config::Config, exec::StatusCode};

#[derive(Debug)]
pub struct Unknown {
    pub name: String,
    cfg: Config,
}

impl Unknown {
    #[must_use]
    /// Creates a new [`Unknown`] package manager with the given name.
    pub fn new(name: &str) -> Self {
        Unknown {
            name: format!("unknown package manager: {}", name),
            cfg: Config::default(),
        }
    }
}

#[async_trait]
impl Pm for Unknown {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        &self.name
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    async fn code(&self) -> StatusCode {
        0
    }

    async fn set_code(&self, _to: StatusCode) {}
}
