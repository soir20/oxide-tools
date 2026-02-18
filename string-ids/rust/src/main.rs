use std::{
    collections::{HashMap, HashSet},
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
    range_start: i32,
    /// Ending ID to brute-force
    #[arg(default_value_t = i32::MAX)]
    range_end: i32,
}

fn build_digits(buf: &mut [u8; 16], mut value: i32) -> usize {
    if value == 0 {
        buf[0] = b'0';
        return 1;
    }

    let mut digit_count = 0;
    while value >= 10 {
        let quotient = value / 10;
        let remainder = value - quotient * 10;
        buf[digit_count] = b'0' + remainder as u8;
        value = quotient;
        digit_count += 1;
    }

    buf[digit_count] = b'0' + value as u8;
    digit_count += 1;

    buf[..digit_count].reverse();
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
    mut remaining: HashSet<u32>,
    mut hash_to_str: HashMap<u32, String>,
    out_path: &str,
    range_start: i32,
    range_end: i32,
) -> io::Result<()> {
    const PREFIX: &[u8] = b"Global.Text.";
    const PREFIX_LEN: usize = 12;

    let mut results = Vec::<(i32, String)>::new();
    let mut digits_buf = [0u8; 16];
    let mut key_buf = [0u8; 32];

    key_buf[..PREFIX_LEN].copy_from_slice(PREFIX);

    for id in range_start..range_end {
        if remaining.is_empty() {
            break;
        }

        let digit_count = build_digits(&mut digits_buf, id);
        key_buf[PREFIX_LEN..PREFIX_LEN + digit_count]
            .copy_from_slice(&digits_buf[..digit_count]);

        let hash = lookup2(&key_buf[..PREFIX_LEN + digit_count], 0);

        if let Some(text) = hash_to_str.remove(&hash) {
            remaining.remove(&hash);
            results.push((id, text));
        }
    }

    results.sort_by_key(|x| x.0);

    let mut writer = BufWriter::new(File::create(out_path)?);
    for (id, text) in results {
        writeln!(writer, "{id}\t{text}")?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let start_time = Instant::now();
    let args = Args::parse();

    println!("Starting…");

    let hash_to_str = load_hashes(&args.input)?;
    let remaining: HashSet<u32> = hash_to_str.keys().copied().collect();

    brute_force(
        remaining,
        hash_to_str,
        &args.output,
        args.range_start,
        args.range_end,
    )?;

    println!("Done. Elapsed: {:.2?}", start_time.elapsed());
    Ok(())
}
