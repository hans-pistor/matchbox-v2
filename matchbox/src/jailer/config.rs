use derive_builder::Builder;
use std::path::PathBuf;

#[derive(Copy, Clone, Debug)]
pub struct JailerGid(u32);

impl Default for JailerGid {
    fn default() -> Self {
        Self(users::get_effective_gid())
    }
}

impl From<&JailerGid> for u32 {
    fn from(value: &JailerGid) -> Self {
        value.0
    }
}

#[derive(Clone, Debug)]
pub struct JailerUid(u32);
impl Default for JailerUid {
    fn default() -> Self {
        Self(users::get_effective_uid())
    }
}

impl From<&JailerUid> for u32 {
    fn from(value: &JailerUid) -> Self {
        value.0
    }
}

#[derive(Builder, Clone, Debug)]
#[builder(setter(into))]
pub struct JailerConfig {
    /// Path to the  jailer binary
    pub jailer_path: PathBuf,

    /// VM identification string, alphanumeric + hyphens
    pub id: String,

    /// Path to the binary jailer will exec, generally firecracker
    pub exec_file: PathBuf,

    /// Base directory of the chrooted process
    pub chroot_base_dir: PathBuf,

    /// Path of the network namespace
    pub netns: PathBuf,

    /// gid jailer will switch to when executed the exec_file process
    #[builder(default)]
    pub gid: JailerGid,

    /// uid jailer will switch to when executed the exec_file process
    #[builder(default)]
    pub uid: JailerUid,
}
