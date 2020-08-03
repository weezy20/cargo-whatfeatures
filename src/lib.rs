/*! whatfeatures

print out features and dependencies for a specific crate
*/

mod args;
mod client;
mod features;
mod printer;
mod registry;
mod util;

#[doc(inline)]
pub use client::{Client, Version};

#[doc(inline)]
pub use registry::{Crate, Registry, YankState};

#[doc(inline)]
pub use args::{Args, PkgId};

#[doc(inline)]
pub use printer::*;

// TODO move all of this to another module

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
/// An error for when the crate is in 'offline mode'
pub enum OfflineError {
    /// Cannot list versions
    List,
    /// Cannot get the latest version
    Latest,
    /// Crate wasn't cached locally, cannot look it up
    CacheMiss,
}

impl OfflineError {
    /// Converts this type to an Error type
    pub fn to_error(&self) -> anyhow::Error {
        let err = match self {
            Self::List => {
                "must be able to connect to https://crates.io to list versions"
            }
            Self::Latest =>{
                "must be able to connect to https://crates.io to get the latest version"
            } ,
            Self::CacheMiss => {
                "crate not found in local registry or cache. must be able to connect to https://crates.io to fetch it"
            },
        };
        anyhow::anyhow!(err)
    }
}

#[derive(Debug)]
/// Lookup result
pub enum Lookup {
    /// A partial lookup -- this has to cache the crate
    Partial(Version),
    /// A local workspace
    Workspace(features::Workspace),
}

/// Find this 'pkgid'
pub fn lookup(pkg_id: &PkgId, client: &Option<Client>) -> anyhow::Result<Lookup> {
    match pkg_id {
        // lookup the latest version
        PkgId::Remote { name, semver } => {
            let client = client
                .as_ref()
                .ok_or_else(|| OfflineError::Latest.to_error())?;

            let pkg = match semver {
                Some(semver) => client.get_version(name, semver),
                None => client.get_latest(name),
            }
            .map_err(|_| cannot_find(pkg_id))?;

            Ok(Lookup::Partial(pkg))
        }

        // otherwise load it from the local path
        PkgId::Local(path) => Crate::from_path(path).map(Lookup::Workspace),
    }
}

fn cannot_find(pkg_id: &PkgId) -> anyhow::Error {
    anyhow::anyhow!(
        "cannot find a crate matching '{}'. maybe it was yanked?",
        pkg_id
    )
}
