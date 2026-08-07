// build.rs

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("consts.rs");

    #[rustfmt::skip]
    const S: [u32; 64] = [
        7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,
        5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,
        4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,
        6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,
    ];

    let k: [u32; 64] = (0..64)
        .map(|i| (((i as f64) + 1_f64).sin().abs() * (u32::MAX as f64 + 1_f64)) as u32)
        .collect::<Vec<u32>>()
        .try_into()
        .unwrap();

    fs::write(
        &dest_path,
        format!(
            "
const S: [u32; 64] = {:?};
const K: [u32; 64] = {:?};
const CONSTANTS: [[u32; 64]; 2] = [S, K];

            ",
            S, k
        ),
    )
    .unwrap();
    println!("cargo::rerun-if-changed=build.rs");
}
