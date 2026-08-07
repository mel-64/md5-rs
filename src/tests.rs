#[cfg(test)]
mod tests {
    use std::io::{self, BufReader, Error, Seek, Write};
    use tempfile::tempfile;

    fn get_byte_hash(bytes: &[u8]) -> Result<u128, Error> {
        let mut tempfile = tempfile()?;
        tempfile.write(bytes)?;
        tempfile.flush()?;
        tempfile.seek(io::SeekFrom::Start(0))?;
        let mut reader = BufReader::new(tempfile);
        Ok(crate::get_hash(&mut reader))
    }

    #[test]
    fn txt() {
        assert_eq!(
            "9e107d9d372bb6826bd81d3542a419d6",
            format!(
                "{:032x}",
                get_byte_hash("The quick brown fox jumps over the lazy dog".as_bytes()).unwrap()
            )
        );
    }

    #[test]
    fn bytes() {
        assert_eq!(
            "3cc897735501f9b83dcad3436d917899",
            format!(
                "{:032x}",
                get_byte_hash(&[
                    0x90, 0xdc, 0xe0, 0x5c, 0xfc, 0x77, 0x26, 0x67, 0x4f, 0x60, 0x61, 0x70, 0xc8,
                    0x6d, 0xfc, 0x9b
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn nullbyte() {
        assert_eq!(
            "d41d8cd98f00b204e9800998ecf8427e",
            format!("{:032x}", get_byte_hash(&[]).unwrap())
        );
    }
}
