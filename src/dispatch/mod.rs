//! This module handles argument parsing and environment detection. The [`Opts`] structure is generated by [`clap`],
//! which then generates the correct [`pm::Pm`] trait object according to the environmental context,
//! and then call the corresponding trait method.

mod cmd;
pub mod config;

pub use self::{cmd::Opts, config::Config};
use crate::{exec::is_exe, pm::*};

/// Detects the name of the package manager to be used in auto dispatch.
pub fn detect_pm_str<'s>() -> &'s str {
    let pairs: &[(&str, &str)] = match () {
        _ if cfg!(target_os = "windows") => &[("scoop", ""), ("choco", "")],

        _ if cfg!(target_os = "macos") => &[
            ("brew", "/usr/local/bin/brew"),
            ("port", "/opt/local/bin/port"),
        ],

        _ if cfg!(target_os = "linux") => &[
            ("apk", "/sbin/apk"),
            ("apt", "/usr/bin/apt"),
            ("emerge", "/usr/bin/emerge"),
            ("dnf", "/usr/bin/dnf"),
            ("zypper", "/usr/bin/zypper"),
        ],

        _ => &[],
    };

    pairs
        .iter()
        .find_map(|(name, path)| is_exe(name, path).then(|| *name))
        .unwrap_or("unknown")
}

impl From<Config> for Box<dyn Pm> {
    /// Generates the `Pm` instance according it's name, feeding it with the current `Config`.
    fn from(cfg: Config) -> Self {
        // If the `Pm` to be used is not stated in any config,
        // we should fall back to automatic detection.
        let pm = cfg.default_pm.as_deref().unwrap_or_else(detect_pm_str);

        #[allow(clippy::match_single_binding)]
        match pm {
            // Chocolatey
            "choco" => Chocolatey { cfg }.boxed(),

            // Scoop
            "scoop" => Scoop { cfg }.boxed(),

            // Homebrew/Linuxbrew
            "brew" => Homebrew { cfg }.boxed(),

            // Macports
            "port" if cfg!(target_os = "macos") => Macports { cfg }.boxed(),

            // Portage for Gentoo
            "emerge" => Portage { cfg }.boxed(),

            // Apk for Alpine
            "apk" => Apk { cfg }.boxed(),

            // Apt for Debian/Ubuntu/Termux (new versions)
            "apt" => Apt { cfg }.boxed(),

            // Dnf for RedHat
            "dnf" => Dnf { cfg }.boxed(),

            // Zypper for SUSE
            "zypper" => Zypper { cfg }.boxed(),

            // * External Package Managers *

            // Conda
            "conda" => Conda { cfg }.boxed(),

            // Pip
            "pip" => Pip {
                cmd: "pip".into(),
                cfg,
            }
            .boxed(),

            "pip3" => Pip {
                cmd: "pip3".into(),
                cfg,
            }
            .boxed(),

            // Tlmgr
            "tlmgr" => Tlmgr { cfg }.boxed(),

            // Test-only mock package manager
            #[cfg(test)]
            "mockpm" => {
                use self::cmd::tests::MockPm;
                MockPm { cfg }.boxed()
            }

            // Unknown package manager X
            x => Unknown::new(x).boxed(),
        }
    }
}
