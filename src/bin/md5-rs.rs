use md5_rs::*;
use std::io::{self, Write};
use std::process::exit;

fn main() {
    let input_file_path = std::env::args().nth(1).unwrap_or("-".to_owned());

    let mut reader = match create_reader(&input_file_path) {
        Ok(r) => r,
        Err(e) => {
            let _ = writeln!(
                io::stderr(),
                "{}: {}: {}",
                std::env::args().next().unwrap(),
                input_file_path,
                e
            );
            exit(1);
        }
    };

    let res = get_hash(&mut reader);

    let _ = writeln!(io::stdout(), "{res:032x}  {input_file_path}");
}
