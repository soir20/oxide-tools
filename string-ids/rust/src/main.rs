use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    time::Instant,
};

use clap::Parser;
use jenkins_lookup2::lookup2;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Brute force script to recover string IDs from hashes in SOE .dat locale files"
)]
struct Args {
    /// Input .dat file (filename or path, relative to where you run the tool)
    input: String,
    /// Output file to write recovered IDs into (filename or path)
    output: String,
    /// Starting ID to brute-force
    #[arg(default_value_t = 0)]
    range_start: u32,
    /// Ending ID to brute-force
    #[arg(default_value_t = u32::MAX)]
    range_end: u32,
}

static DIGITS_4: [[u8; 4]; 10000] = {
    let mut table = [[0u8; 4]; 10000];
    let mut i = 0;
    while i < 10000 {
        table[i][0] = b'0' + (i / 1000) as u8;
        table[i][1] = b'0' + ((i / 100) % 10) as u8;
        table[i][2] = b'0' + ((i / 10) % 10) as u8;
        table[i][3] = b'0' + (i % 10) as u8;
        i += 1;
    }
    table
};

fn build_digits(buf: &mut [u8; 16], mut value: u32) -> usize {
    if value < 10 {
        buf[0] = b'0' + value as u8;
        return 1;
    }

    let mut write_index = 16;

    while value >= 10000 {
        let remainder = (value % 10000) as usize;
        value /= 10000;
        write_index -= 4;
        buf[write_index..write_index + 4].copy_from_slice(&DIGITS_4[remainder]);
    }

    if value < 10 {
        write_index -= 1;
        buf[write_index] = b'0' + value as u8;
    } else if value < 100 {
        write_index -= 2;
        let digits = DIGITS_4[value as usize];
        buf[write_index] = digits[2];
        buf[write_index + 1] = digits[3];
    } else if value < 1000 {
        write_index -= 3;
        let digits = DIGITS_4[value as usize];
        buf[write_index] = digits[1];
        buf[write_index + 1] = digits[2];
        buf[write_index + 2] = digits[3];
    } else {
        write_index -= 4;
        buf[write_index..write_index + 4].copy_from_slice(&DIGITS_4[value as usize]);
    }

    let digit_count = 16 - write_index;
    buf.copy_within(write_index..16, 0);
    digit_count
}

fn load_hashes(path: &str) -> io::Result<HashMap<u32, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }

        if let Ok(hash) = parts[0].parse::<u32>() {
            map.insert(hash, parts[2].to_string());
        }
    }

    Ok(map)
}

fn brute_force(
    hash_to_str: HashMap<u32, String>,
    out_path: &str,
    range_start: u32,
    range_end: u32,
) -> io::Result<()> {
    const PREFIX: &[u8] = b"Global.Text.";
    const PREFIX_LEN: usize = 12;

    let mut results = Vec::with_capacity(hash_to_str.len());
    let mut digits_buf = [0u8; 16];
    let mut key_buf = [0u8; 32];

    key_buf[..PREFIX_LEN].copy_from_slice(PREFIX);

    for id in range_start..range_end {
        let digit_count = build_digits(&mut digits_buf, id);
        key_buf[PREFIX_LEN..PREFIX_LEN + digit_count].copy_from_slice(&digits_buf[..digit_count]);

        let hash = lookup2(&key_buf[..PREFIX_LEN + digit_count], 0);

        if let Some(text) = hash_to_str.get(&hash) {
            results.push((id, text.clone()));
        }
    }

    let mut writer = BufWriter::new(File::create(out_path)?);
    for (id, text) in results {
        writeln!(writer, "{id}\t{text}")?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let start_time = Instant::now();
    let args = Args::parse();

    println!("Generating…");

    let hash_to_str = load_hashes(&args.input)?;

    brute_force(hash_to_str, &args.output, args.range_start, args.range_end)?;

    println!("Complete!\nElapsed: {:.2?}", start_time.elapsed());
    Ok(())
}
