use std::io::Read;
use num::PrimInt;

pub trait ShaiyaIo {

    /// Reads a C-style string from a fixed length array.
    ///
    /// # Arguments
    /// * `length`  - The length of the string.
    fn read_fixed_length_string<T: PrimInt>(&mut self, length: T) -> anyhow::Result<String>;
}

impl <T> ShaiyaIo for T where T: Read {

    /// Reads a C-style string from a fixed length array.
    ///
    /// # Arguments
    /// * `length`  - The length of the string.
    fn read_fixed_length_string<N: PrimInt>(&mut self, length: N) -> anyhow::Result<String> {
        let mut data: Vec<u8> = vec![0; length.to_usize().unwrap()];
        let payload = data.as_mut_slice();
        self.read_exact(payload)?;

        // If the last element is a nullpointer, remove it.
        if let Some(last) = payload.last() {
            if *last == 0 {
                data.pop();
            }
        }

        Ok(String::from_utf8_lossy(&data)
            .to_string())
    }
}