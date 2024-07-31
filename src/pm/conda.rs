#![doc = doc_self!()]

use std::sync::LazyLock;

use async_trait::async_trait;
use futures::prelude::*;
use indoc::indoc;
use tap::prelude::*;

use super::{Pm, PmHelper, PmMode, PromptStrategy, Strategy};
use crate::{config::Config, error::Result, exec::Cmd};

macro_rules! doc_self {
    () => {
        indoc! {"
            The [Conda Package Manager](https://conda.io/).
        "}
    };
}
use doc_self;

#[doc = doc_self!()]
#[derive(Debug)]
pub struct Conda {
    cfg: Config,
}

static STRAT_PROMPT: LazyLock<Strategy> = LazyLock::new(|| Strategy {
    prompt: PromptStrategy::native_no_confirm(["-y"]),
    ..Strategy::default()
});

impl Conda {
    #[must_use]
    #[allow(missing_docs)]
    pub const fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Pm for Conda {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        "conda"
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }

    /// Q generates a list of installed packages.
    async fn q(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        if kws.is_empty() {
            self.run(Cmd::new(["conda", "list"]).flags(flags)).await
        } else {
            self.qs(kws, flags).await
        }
    }

    /// Qo queries the package which provides FILE.
    async fn qo(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["conda", "package", "--which"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Qs searches locally installed package for names or descriptions.
    // According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
    // when including multiple search terms, only packages with descriptions
    // matching ALL of those terms are returned.
    async fn qs(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.search_regex(Cmd::new(["conda", "list"]).flags(flags), kws)
            .await
    }

    /// R removes a single package, leaving all of its dependencies installed.
    async fn r(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["conda", "remove"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// S installs one or more packages by name.
    async fn s(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["conda", "install"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Sc removes all the cached packages that are not currently installed, and
    /// the unused sync database.
    async fn sc(&self, _kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["conda", "clean", "--all"])
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Si displays remote package information: name, version, description, etc.
    async fn si(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["conda", "search", "--info"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run(cmd))
            .await
    }

    /// Ss searches for package(s) by searching the expression in name,
    /// description, short description.
    async fn ss(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        stream::iter(kws)
            .map(|s| Ok(format!("*{s}*")))
            .try_for_each(|kw| self.run(Cmd::new(["conda", "search"]).kws([kw]).flags(flags)))
            .await
    }

    /// Su updates outdated packages.
    async fn su(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        Cmd::new(["conda", "update", "--all"])
            .kws(kws)
            .flags(flags)
            .pipe(|cmd| self.run_with(cmd, PmMode::default(), &STRAT_PROMPT))
            .await
    }

    /// Suy refreshes the local package database, then updates outdated
    /// packages.
    async fn suy(&self, kws: &[&str], flags: &[&str]) -> Result<()> {
        self.su(kws, flags).await
    }
}
