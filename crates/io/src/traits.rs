use crate::{errors::ResourceError, paths::AppPaths};

pub trait Readable {
    /// The associated output type that will be returned when reading the resource.
    type Output;

    /// Reads the resource represented by the implementing type.
    ///
    /// # Arguments
    /// * `paths` - A reference to `AppPaths` which provides the necessary paths for reading the resource.
    /// * `max_file_size` - An optional maximum file size limit for the resource being read. If the resource exceeds this size, an error will be returned.
    ///
    /// # Returns
    /// A `Result` containing the output of the read operation or a `ResourceError` if the operation fails.
    fn read(self, paths: &AppPaths, max_file_size: Option<u64>) -> Result<Self::Output, ResourceError>;
}

pub trait Writable {
    /// Writes the provided data to the resource represented by the implementing type.
    ///
    /// # Arguments
    /// * `data` - The data to be written, which can be any type that implements `AsRef<[u8]>`.
    /// * `paths` - A reference to `AppPaths` which provides the necessary paths for writing the resource.
    ///
    /// # Returns
    /// A `Result` indicating success or containing a `ResourceError` if the operation fails.
    fn write<C: AsRef<[u8]>>(self, data: C, paths: &AppPaths) -> Result<(), ResourceError>;
}
