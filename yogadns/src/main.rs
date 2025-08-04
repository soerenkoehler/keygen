use std::{cmp::max, sync::mpsc::channel, thread::spawn};

use rand::random_range;

fn main() {
    create_key(0, 3, 0, 100, 0, 0, 0, 0);
}

fn create_key(
    threads: i32,
    prod1: i32,
    prod2: i32,
    v1: i32,
    v2: i32,
    year: i32,
    month: i32,
    unknown: i32,
) {
    const MAGIC_1: i32 = i32::from_be_bytes([0x12, 0x34, 0x56, 0x78]);
    const MAGIC_2: i32 = i32::from_be_bytes([0x87, 0x65, 0x43, 0x21]);

    let info3 =
        ((prod1 & 0x7ff) << 21) | ((prod2 & 0x1f) << 16) | ((v1 & 0x7ff) << 5) | (v2 & 0x1f);
    let info4 = (unknown & 0xffff)
        | if year > 0 {
            (((year - 2000) * 12 + month) & 0xffff) << 16
        } else {
            0
        };

    let search_filler = move |sub_key1| guess_key_filler(sub_key1, info3, info4);
    let output_key = |(sub_key1, filler)| {
        let info1 = sub_key1 ^ (sub_key1 << 7);

        let part1 = encode_sub_key(info3 ^ info1 ^ MAGIC_1, 7);
        let part2 = encode_sub_key(info4 ^ info1 ^ MAGIC_2, 7);
        let part3 = encode_sub_key(filler, 5);
        let part4 = encode_sub_key(sub_key1, 5);

        println!(
            "{}",
            [
                &part1[..5],
                "-",
                &part1[5..7],
                &part2[..3],
                "-",
                &part2[3..7],
                &part1[2..3],
                "-",
                &part3,
                "-",
                &part4,
            ]
            .concat()
        );
    };

    if threads > 0 { // run inparallel
        let (tx, rx) = channel();
        let batch_size = (1 << 25) / threads;
        (0..threads)
            .map(|thread_index| {
                let start = thread_index * batch_size;
                let end = max(thread_index * batch_size + batch_size, 1 << 25);
                let tx_clone = tx.clone();
                spawn(move || {
                    (start..end).flat_map(search_filler).for_each(|pair| {
                        let _ = tx_clone.send(pair);
                    });
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|thread| {
                let _ = thread.join();
            });
        rx.iter().for_each(output_key);
    } else { // pick a random start value for sub_key1 and return the first 12 results
        (random_range(0..1 << 25)..)
            .flat_map(|sub_key1| guess_key_filler(sub_key1, info3, info4))
            .take(12)
            .for_each(output_key)
    }
}

fn encode_sub_key(mut value: i32, mut length: i32) -> String {
    let mut subkey = String::new();
    while length > 0 {
        let bits = value as u8 & 0x1f;
        let char_code = bits + if bits <= 9 { 0x30 } else { 0x37 };
        subkey.push(char_code as char);
        value >>= 5;
        length -= 1;
    }
    subkey
}

fn guess_key_filler(sub_key1: i32, info3: i32, info4: i32) -> impl Iterator<Item = (i32, i32)> {
    let seed = update_checksum(update_checksum(-1, info3), info4);
    (0..1 << 25)
        .map(move |filler| (filler, update_checksum(seed, filler) & 0x1ffffff))
        .filter_map(move |(filler, check)| {
            if sub_key1 == check {
                Some((sub_key1, filler))
            } else {
                None
            }
        })
}

fn update_checksum(seed: i32, value: i32) -> i32 {
    let mut check = seed;
    for byte in [
        (value >> 0) & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
    ] {
        check ^= byte << 24;
        for _ in 0..8 {
            if check >= 0 {
                check = check << 1;
            } else {
                check = (check << 1) ^ 0x4C11DB7
            }
        }
    }
    check
}
