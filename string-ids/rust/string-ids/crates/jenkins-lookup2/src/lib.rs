fn mix(a: &mut u32, b: &mut u32, c: &mut u32) {
    *a = a.wrapping_sub(*b); *a = a.wrapping_sub(*c); *a ^= *c >> 13;
    *b = b.wrapping_sub(*c); *b = b.wrapping_sub(*a); *b ^= *a << 8;
    *c = c.wrapping_sub(*a); *c = c.wrapping_sub(*b); *c ^= *b >> 13;

    *a = a.wrapping_sub(*b); *a = a.wrapping_sub(*c); *a ^= *c >> 12;
    *b = b.wrapping_sub(*c); *b = b.wrapping_sub(*a); *b ^= *a << 16;
    *c = c.wrapping_sub(*a); *c = c.wrapping_sub(*b); *c ^= *b >> 5;

    *a = a.wrapping_sub(*b); *a = a.wrapping_sub(*c); *a ^= *c >> 3;
    *b = b.wrapping_sub(*c); *b = b.wrapping_sub(*a); *b ^= *a << 10;
    *c = c.wrapping_sub(*a); *c = c.wrapping_sub(*b); *c ^= *b >> 15;
}

pub fn lookup2(key: &[u8], initval: u32) -> u32 {
    let mut a: u32 = 0x9e3779b9;
    let mut b: u32 = 0x9e3779b9;
    let mut c: u32 = initval;

    let mut len = key.len() as u32;
    let mut k = 0usize;

    while len >= 12 {
        a = a.wrapping_add(u32::from_le_bytes([
            key[k], key[k + 1], key[k + 2], key[k + 3],
        ]));
        b = b.wrapping_add(u32::from_le_bytes([
            key[k + 4], key[k + 5], key[k + 6], key[k + 7],
        ]));
        c = c.wrapping_add(u32::from_le_bytes([
            key[k + 8], key[k + 9], key[k + 10], key[k + 11],
        ]));

        mix(&mut a, &mut b, &mut c);

        k += 12;
        len -= 12;
    }

    c = c.wrapping_add(key.len() as u32);

    match len {
        11 => c = c.wrapping_add((key[k + 10] as u32) << 24),
        10 => c = c.wrapping_add((key[k + 9] as u32) << 16),
        9  => c = c.wrapping_add((key[k + 8] as u32) << 8),
        _ => {}
    }

    match len {
        8 | 9 | 10 | 11 => b = b.wrapping_add((key[k + 7] as u32) << 24),
        _ => {}
    }
    match len {
        7 | 8 | 9 | 10 | 11 => b = b.wrapping_add((key[k + 6] as u32) << 16),
        _ => {}
    }
    match len {
        6 | 7 | 8 | 9 | 10 | 11 => b = b.wrapping_add((key[k + 5] as u32) << 8),
        _ => {}
    }
    match len {
        5 | 6 | 7 | 8 | 9 | 10 | 11 => b = b.wrapping_add(key[k + 4] as u32),
        _ => {}
    }

    match len {
        4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 => a = a.wrapping_add((key[k + 3] as u32) << 24),
        _ => {}
    }
    match len {
        3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 => a = a.wrapping_add((key[k + 2] as u32) << 16),
        _ => {}
    }
    match len {
        2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 => a = a.wrapping_add((key[k + 1] as u32) << 8),
        _ => {}
    }
    match len {
        1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 => a = a.wrapping_add(key[k] as u32),
        _ => {}
    }

    mix(&mut a, &mut b, &mut c);
    c
}
