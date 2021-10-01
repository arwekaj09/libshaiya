use anyhow::anyhow;
use std::fs::{File, OpenOptions};
use std::path::Path;
use file::{SFolder, SFile};
use std::io::{Cursor, Read, Seek, SeekFrom};
use crate::utils::ShaiyaIo;
use byteorder::{ReadBytesExt, LittleEndian};
use std::collections::VecDeque;

mod file;

/// The magic value of the SAH file.
pub const SAH_MAGIC_VALUE: &'static str = "SAH";

/// The default name of the Shaiya archive header file.
pub const DEFAULT_HEADER_NAME: &'static str = "data.sah";

/// The default name of the Shaiya archive data file.
pub const DEFAULT_ARCHIVE_NAME: &'static str = "data.saf";

/// The default name of the root data folder.
pub const DEFAULT_ROOT_NAME: &'static str = "data";

/// An `archive` is a binary format which contains a header ("SAH"), and a data file ("SAF").
///
/// The `header` of the archive represents a virtual filesystem - folders and files. Each entry in
/// the header contains a name, offset, and length, which points to a chunk of data in the archive file.
///
/// The `data file` is just a contiguous block of data, containing the data of every file in the archive.
/// This allow for random access to files.
pub struct Archive {
    header_file: File,
    data_file: File,
    pub root: SFolder,
}

impl Archive {

    /// Creates a new Shaiya archive, with no data. This is useful for building an archive from a
    /// collection of files, or from initialising an archive from patches alone.
    ///
    /// # Arguments
    /// * `path`    - The path to write the archive to.
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        // The path to the header and data file.
        let header_file_path = path.join(Path::new(DEFAULT_HEADER_NAME));
        let data_file_path = path.join(Path::new(DEFAULT_ARCHIVE_NAME));

        // If the files already exist, we should return an error - we don't want to overwrite everything.
        if header_file_path.exists() {
            return Err(anyhow!("Header file already exists."));
        } else if data_file_path.exists() {
            return Err(anyhow!("Data file already exists."));
        }

        Ok(Self {
            header_file:    File::create(header_file_path)?,
            data_file:      File::create(data_file_path)?,
            root:           SFolder::new(DEFAULT_ROOT_NAME.to_owned()),
        })
    }

    /// Opens an existing Shaiya archive.
    ///
    /// # Arguments
    /// * `header_path` - The path to the header file.
    /// * `data_path`   - The path to the data file.
    pub fn open(header_path: &Path, data_path: &Path) -> anyhow::Result<Self> {
        let header_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(header_path)?;
        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(data_path)?;
        let root = SFolder::new(DEFAULT_ROOT_NAME.to_owned());

        let mut archive = Self { header_file, data_file, root };
        archive.parse()?;
        Ok(archive)
    }

    /// Parses the archive files, and populates the virtual filesystem.
    pub fn parse(&mut self) -> anyhow::Result<()> {
        // Read the contents of the header file.
        let mut header_data = Vec::new();
        self.header_file.read_to_end(&mut header_data)?;

        // Create a cursor to read from.
        let mut cursor = Cursor::new(header_data);

        // Validate the header's magic value.
        let name = cursor.read_fixed_length_string(SAH_MAGIC_VALUE.len())?;
        if name != SAH_MAGIC_VALUE {
            return Err(anyhow!("Invalid SAH magic value: {} - expected {}", name, SAH_MAGIC_VALUE))
        }

        cursor.seek(SeekFrom::Current(4))?;     // Skip the next 4 bytes (they don't seem to do anything).
        cursor.seek(SeekFrom::Current(4))?;     // The total number of files (the client doesn't even use this).
        cursor.seek(SeekFrom::Current(40))?;    // Skip the next 40 bytes (again - they don't seem to do anything).

        // Read the name of the root folder.
        let root_name_len = cursor.read_u32::<LittleEndian>()?;
        let root_name = cursor.read_fixed_length_string(root_name_len)?;

        // Initialise an empty root folder, and parse it's contents.
        self.root = SFolder::new(root_name);
        self.root.parse(&mut cursor)
    }

    /// Gets the data for a specified file.
    ///
    /// # Arguments
    /// * `file`    - The file to get the data for.
    pub fn file_data(&mut self, file: &SFile) -> anyhow::Result<Vec<u8>> {
        // Create a vector to store the data.
        let mut data: Vec<u8> = vec![0; file.length as usize];
        let slice = data.as_mut_slice();

        // Seek to a position in the data file and read the file's data.
        self.data_file.seek(SeekFrom::Start(file.offset))?;
        self.data_file.read(slice)?;
        Ok(data)
    }

    /// Gets a file for a given path.
    ///
    /// # Arguments
    /// * `path`    - The path to the file.
    pub fn get(&mut self, path: &str) -> Option<SFile> {
        // Split the path into parts.
        let mut parts: VecDeque<&str> = path.split("/").collect();
        self.root.get(&mut parts)
    }
}

#[cfg(test)]
mod tests {
    use crate::archive::Archive;
    use std::path::Path;

    /// Tests the validity of a known-good Ep5 archive.
    #[test]
    fn test_ep5_archive() -> anyhow::Result<()> {
        // The path to the header and data file.
        let header_file = Path::new("ep5/data.sah");
        let data_file = Path::new("ep5/data.saf");

        // Initialise the archive.
        let mut archive = Archive::open(header_file, data_file)?;

        // Test that we can find some key files.
        assert!(archive.get("cl.tga").is_some());
        assert!(archive.get("sysmsg-uni.txt").is_some());
        assert!(archive.get("item/item.sdata").is_some());

        // Test that we can get the data for a file.
        let skill_sdata = archive.get("character/skill.sdata");
        assert!(skill_sdata.is_some());
        let file = skill_sdata.unwrap();
        let data = archive.file_data(&file)?;
        assert!(!data.is_empty());

        Ok(())
    }
}