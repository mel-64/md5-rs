use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, Error, ErrorKind, Read, Write};
use std::process::exit;

mod tests;

// This writes the global constant CONSTANTS[u32; 2] containing `S` and `K`.
// It has to be done this way, as K needs iteration to be constructed and can therefore not be
// inlined if it's not included in the constructed form.
// This saves a few CPU cycles and therefore should speed up the execution speed marginally.
include!(concat!(env!("OUT_DIR"), "/consts.rs"));

#[allow(arithmetic_overflow)]
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

fn get_hash(reader: &mut BufReader<File>) -> u128 {
    let mut state: [u32; 4] = [0x67452301, 0xefcdab89_u32, 0x98badcfe_u32, 0x10325476_u32];

    let mut input = ByteArray::new(vec![0_u8; 0]);

    reader.read_to_end(&mut input.data).unwrap(); // Todo: chunked read

    prep_input(&mut input); // Todo: pad and append on last chunk

    for b in input.data.windows(64).step_by(64) {
        // Todo: use chunks from BufReader
        process_block(b.try_into().unwrap(), &mut state, CONSTANTS);
    }
    let state: [u8; 16] = state
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    u128::from_be_bytes(state)
}

struct ByteArray {
    data: Vec<u8>,
}

struct ByteArrayIter<'a> {
    data: &'a ByteArray,
    index: usize,
}

impl ByteArray {
    fn new(data: Vec<u8>) -> Self {
        ByteArray { data }
    }

    fn iter(&self) -> ByteArrayIter<'_> {
        ByteArrayIter {
            data: self,
            index: 0,
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn append(&mut self, other: &mut Vec<u8>) {
        self.data.append(other)
    }

    fn push(&mut self, other: u8) {
        self.data.push(other)
    }
}

impl<'a> Iterator for ByteArrayIter<'a> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.data.len() {
            self.index += 1;
            Some(&self.data.data[self.index - 1])
        } else {
            None
        }
    }
}

impl fmt::Display for ByteArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|b| format!("{b:#04x}"))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

fn create_reader(file_path: &str) -> std::io::Result<BufReader<File>> {
    let file = if file_path == "-" {
        File::open("/dev/stdin")?
    } else {
        File::open(file_path)?
    };
    if file.metadata()?.is_dir() {
        return Err(Error::new(ErrorKind::IsADirectory, "Is a directory"));
    }
    let reader = BufReader::with_capacity(64, file); // 64 byte => 512 bit
    Ok(reader)
}

fn prep_input(input: &mut ByteArray) {
    let len = (input.len() as u128 * 8) % 2_u128.pow(64);

    input.push(0x80_u8);

    let padding_amount: usize = (56 - (input.len() % 64) + 64) % 64; // Can probs be optimized, lol
    input.append(&mut vec![0_u8; padding_amount]);
    input.append(&mut u128::to_le_bytes(len).to_vec());
}

fn process_block(block: &[u8; 64], state: &mut [u32; 4], constants: [[u32; 64]; 2]) {
    // Block is 64 bytes long
    let old_state: [u32; 4] = *state;
    let [mut a, mut b, mut c, mut d] = [state[0], state[1], state[2], state[3]];
    let [s, k] = constants;
    let chunks: Vec<u32> = (0..block.len())
        .step_by(4)
        .map(|i| u32::from_le_bytes(block[i..i + 4].try_into().unwrap()))
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

    state[0] = old_state[0] + a;
    state[1] = old_state[1] + b;
    state[2] = old_state[2] + c;
    state[3] = old_state[3] + d;
}
