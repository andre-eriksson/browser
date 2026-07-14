use storage::Directory;

use crate::errors::ResourceError;

pub trait Loadable {
    type Output;
    fn load_asset(self, dirs: &Directory, max_file_size: Option<u64>) -> Result<Self::Output, ResourceError>;
}

pub trait Writable {
    fn write_asset<C: AsRef<[u8]>>(self, data: C, dirs: &Directory) -> Result<(), ResourceError>;
}
