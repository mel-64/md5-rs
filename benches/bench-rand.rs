use criterion::{Criterion, criterion_group, criterion_main};
use getrandom;
use md5_rs::get_hash;
use std::error;
use std::hint::black_box;
use std::io::{BufReader, Seek, Write};
use std::time::Duration;
use tempfile::NamedTempFile;

fn get_random_file(size: usize) -> Result<NamedTempFile, Box<dyn error::Error>> {
    let mut tempfile = NamedTempFile::new_in("/var/tmp").unwrap();
    let mut random = &mut vec![0_u8; size];
    getrandom::fill(&mut random)?;
    tempfile.write(&random)?;
    tempfile.flush()?;
    Ok(tempfile)
}

fn rand_benches(c: &mut Criterion) {
    let sizes = [
        FileSize::new(128 << 1),  // 128 B
        FileSize::new(4 << 10),   // 4 KiB
        FileSize::new(512 << 10), // 512 KiB
        FileSize::new(4 << 20),   // 4 MiB
        FileSize::new(128 << 20), // 128 MiB
    ];
    for s in sizes {
        let named_temp_file = get_random_file(s.size).unwrap();
        let path_string = &named_temp_file.path().display().to_string();
        let file = named_temp_file.reopen().unwrap();
        let mut reader = BufReader::new(file);
        println!("Path: {}", path_string);

        let name = format!("Rand {}", s);
        c.bench_function(&name, |b| {
            b.iter(|| {
                black_box(get_hash(&mut reader));
                reader.rewind()
            })
        });
    }
}

struct FileSize {
    size: usize,
}

impl std::fmt::Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format_string = match self {
            Self { size: ..1024 } => format!("{} B", self.size),
            Self { size: ..1048576 } => format!("{} KiB", self.size >> 10),
            Self { size: ..1073741824 } => format!("{} MiB", self.size >> 20),
            _ => format!("{} GiB", self.size >> 30),
        };
        write!(f, "{}", format_string)
    }
}

impl FileSize {
    fn new(size: usize) -> Self {
        Self { size: size }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50).measurement_time(Duration::from_secs(3));
    targets = rand_benches
}
criterion_main!(benches);
