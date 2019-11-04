// Task Description:
//
// Input: A rule file produced by classbench.
// Output: A binary matrix where a row represents the bit-string of a rule.

use csv::{ReaderBuilder, StringRecord};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufReader, Write};

type InputT = Vec<StringRecord>;
type OutputT = Vec<String>;

const WILDCARD: char = '_';

fn ip_to_bitstring(ip: &str) -> String {
    let mut splits = ip.split("/");
    let addr = splits.next().expect("Malformed IP field").trim();
    let masklen = splits
        .next()
        .expect("Malformed IP field")
        .trim()
        .parse::<usize>()
        .expect("Malformed mask value");
    let mut multi = 1;
    let mut ipval = 0;
    for octet in addr.rsplit(".") {
        ipval += octet.parse::<i64>().expect("Malformed IP value") * multi;
        multi <<= 8;
    }
    let maskstr = WILDCARD.to_string().repeat(32 - masklen);
    let mut ipstr = format!("{:032b}", ipval);
    ipstr.truncate(masklen);
    ipstr.push_str(&maskstr);
    ipstr
}

fn port_to_bitstring(port: &str) -> String {
    // TODO: We need to figure out what to do with port ranges like
    // 1024 : 65535. For now, simply take the beginning address.
    let mut splits = port.split(":");
    let begin = splits
        .next()
        .expect("Malformed port field")
        .trim()
        .parse::<i64>()
        .expect("Malformed port value");
    let end = splits
        .next()
        .expect("Malformed port field")
        .trim()
        .parse::<i64>()
        .expect("Malformed port value");
    if begin == 0 && end == 0xFFFF {
        WILDCARD.to_string().repeat(16)
    } else {
        format!("{:016b}", begin)
    }
}

// Parses a hex string that starts with "0x"
fn parse_hex(s: &str) -> i64 {
    let s = s.trim();
    return i64::from_str_radix(&s[2..], 16).expect("Malformed hex value");
}

fn proto_to_bitstring(proto: &str) -> String {
    let mut splits = proto.split("/");
    let proto = parse_hex(splits.next().expect("Mailformed protocol field"));
    let mask = parse_hex(splits.next().expect("Mailformed protocol field"));
    let protostr = format!("{:08b}", proto);
    let maskstr = format!("{:08b}", mask);
    protostr
        .chars()
        .zip(maskstr.chars())
        .map(|(p, m)| if m == '1' { p } else { WILDCARD })
        .collect()
}

fn row_to_bitstring(record: &StringRecord) -> String {
    let srcip = record.get(0).expect("Malformed input: not enough fields");
    let dstip = record.get(1).expect("Malformed input: not enough fields");
    let srcport = record.get(2).expect("Malformed input: not enough fields");
    let dstport = record.get(3).expect("Malformed input: not enough fields");
    let proto = record.get(4).expect("Malformed input: not enough fields");
    format!(
        "{} {} {} {} {}",
        ip_to_bitstring(&srcip[1..]), // srcip starts with @
        ip_to_bitstring(dstip),
        port_to_bitstring(srcport),
        port_to_bitstring(dstport),
        proto_to_bitstring(proto)
    )
}

fn input(filename: &str) -> InputT {
    println!("Reading file {}.", filename);
    let file = File::open(filename).expect("Input file failed to open");
    let mut csv_reader = ReaderBuilder::new()
        .delimiter('\t' as u8)
        .has_headers(false)
        .from_reader(BufReader::new(file));
    let ret: Vec<_> = csv_reader
        .records()
        .map(|res| res.expect("Malformed input"))
        .collect();
    println!("Read {} records.", ret.len());
    ret
}

fn process(input: &InputT) -> OutputT {
    input.par_iter().map(row_to_bitstring).collect()
}

fn output(bitstrings: &OutputT, filename: &str) {
    let mut file = File::create(filename).expect("Output file failed to open");
    for bs in bitstrings {
        write!(file, "{}\n", bs).expect("Failed to write");
    }
}

pub fn run(infile: &str, outfile: &str) {
    println!("==== {} ====", file!());
    let data = input(&infile);
    let result = process(&data);
    output(&result, &outfile);
    println!("{} Done.", file!());
}
