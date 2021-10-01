use std::io::Read;
use byteorder::{ReadBytesExt, LittleEndian};
use crate::utils::ShaiyaIo;
use std::collections::VecDeque;

/// A virtual folder in a Shaiya archive.
pub struct SFolder {
    name: String,
    files: Vec<SFile>,
    folders: Vec<SFolder>
}

/// An entry of file data.
#[derive(Clone)]
pub struct SFile {
    pub name: String,
    pub offset: u64,
    pub length: u64,
    checksum: i32,
}

impl SFolder {

    /// Creates an empty folder.
    ///
    /// # Arguments
    /// * `name`    - The name of the folder.
    pub fn new(name: String) -> Self {
        Self {
            name,
            files:      vec![],
            folders:    vec![]
        }
    }

    /// Parses the contents of this folder from a readable source.
    ///
    /// # Arguments
    /// * `buf` - The readable buffer.
    pub fn parse<T: Read>(&mut self, buf: &mut T) -> anyhow::Result<()> {
        // Read the files.
        let file_qty = buf.read_u32::<LittleEndian>()?;
        for _ in 0..file_qty {
            // Read the name of the file entry.
            let file_name_len = buf.read_u32::<LittleEndian>()?;
            let file_name = buf.read_fixed_length_string(file_name_len)?;

            // Read the file metadata.
            let offset = buf.read_u64::<LittleEndian>()?;
            let length = buf.read_u32::<LittleEndian>()?;
            let checksum = buf.read_i32::<LittleEndian>()?;

            // Add the file to this folder.
            self.files.push(SFile {
                name: file_name,
                offset,
                length: length as u64,
                checksum,
            })
        }

        // Read the sub-directories.
        let folder_qty = buf.read_u32::<LittleEndian>()?;
        for _ in 0..folder_qty {
            // Read the name.
            let name_len = buf.read_u32::<LittleEndian>()?;
            let name = buf.read_fixed_length_string(name_len)?;

            // Create the folder, parse it, and add it to the vector of subdirectories.
            let mut subdirectory = SFolder::new(name);
            subdirectory.parse(buf)?;
            self.folders.push(subdirectory);
        }

        Ok(())
    }

    /// Gets the subdirectories in this folder.
    pub fn subdirectories(&self) -> &Vec<SFolder> {
        &self.folders
    }

    /// Gets the files in this folder.
    pub fn files(&self) -> &Vec<SFile> {
        &self.files
    }

    pub fn get(&self, parts: &mut VecDeque<&str>) -> Option<SFile> {
        // Loop through the parts of the path.
        for part in parts.into_iter() {
            // Look for the file in the local files.
            for file in &self.files {
                if file.name.eq_ignore_ascii_case(part) {
                    return Some(file.clone())
                }
            }

            // Look for the part in the subdirectories.
            for folder in &self.folders {
                if folder.name.eq_ignore_ascii_case(part) {
                    parts.pop_front();
                    return folder.get(parts)
                }
            }
        }

        None
    }
}