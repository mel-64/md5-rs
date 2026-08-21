use std::fs::File;
use std::io::{self, BufReader, Error, ErrorKind, Read, Seek, Write};
use std::process::exit;

mod tests;

// This writes the global constant CONSTANTS[u32; 2] containing `S` and `K`.
// It has to be done this way, as K needs iteration to be constructed and can therefore not be
// inlined if it's not included in the constructed form.
// This saves a few CPU cycles and therefore should speed up the execution speed marginally.
include!(concat!(env!("OUT_DIR"), "/consts.rs"));

#[allow(arithmetic_overflow)]
pub fn get_hash(reader: &mut BufReader<File>) -> u128 {
    let mut state: [u32; 4] = [0x67452301, 0xefcdab89_u32, 0x98badcfe_u32, 0x10325476_u32];
    let mut len: u128 = 0;
    let mut remainder: Vec<u8> = vec![];
    loop {
        let mut res = [0u8; 64];
        let read = reader.read_exact(&mut res);
        match read {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    let _ = reader.rewind();
                    let _ = reader.seek_relative(len as i64);
                    len += reader.read_to_end(&mut remainder).unwrap() as u128;
                    break;
                } else {
                    let _ = write!(io::stderr(), "Unexpected error while reading file: {:?}", e);
                    exit(1);
                }
            }
            Ok(v) => v,
        }
        len += 64;
        process_block(&res, &mut state, CONSTANTS);
    }

    for block in get_footer(remainder, len) {
        process_block(&block, &mut state, CONSTANTS);
    }

    let state: [u8; 16] = state
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    u128::from_be_bytes(state)
}

pub fn create_reader(file_path: &str) -> std::io::Result<BufReader<File>> {
    let file = if file_path == "-" {
        File::open("/dev/stdin")?
    } else {
        File::open(file_path)?
    };
    if file.metadata()?.is_dir() {
        return Err(Error::new(ErrorKind::IsADirectory, "Is a directory"));
    }
    let reader = BufReader::with_capacity(32768, file); // 64 byte => 512 bit
    Ok(reader)
}

fn get_footer(block: Vec<u8>, len: u128) -> Vec<[u8; 64]> {
    let mut res: Vec<[u8; 64]> = vec![];
    let mut block = block.clone();
    let len = (len * 8) as u64;

    block.push(0x80_u8);

    if block.len() > 56 {
        block.resize(64, 0u8);
        res.push(block.try_into().unwrap());
        block = vec![0u8; 56];
    } else {
        block.resize(56, 0u8);
    }
    block.extend(len.to_le_bytes());
    res.push(block.try_into().unwrap());
    res
}

#[inline(always)]
fn process_block(block: &[u8; 64], state: &mut [u32; 4], constants: [[u32; 64]; 2]) {
    // Block is 64 bytes long
    let [mut a, mut b, mut c, mut d] = [state[0], state[1], state[2], state[3]];
    let [s, k] = constants;

    let chunks: Vec<u32> = (0..block.len())
        .step_by(4)
        .into_iter()
        .map(|i| u32::from_le_bytes(block[i..(i + 4)].try_into().unwrap()))
        .collect();

    for i in 0..64 {
        let [mut f, mut g] = match i {
            0..16 => [(b & c) | (!b & d), i],
            16..32 => [(d & b) | (!d & c), 5 * i + 1],
            32..48 => [b ^ c ^ d, 3 * i + 5],
            48..64 => [c ^ (b | !d), 7 * i],
            _ => unreachable!(),
        };
        g %= 16;

        f += a + k[i as usize] + chunks[g as usize];
        a = d; // a=d
        d = c; // d=c
        c = b; // c=b
        b += f.rotate_left(s[i as usize]); // b=b+f.rotate(s[i])
    }

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
}
