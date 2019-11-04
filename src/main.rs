use clap;
// use indicatif::ProgressIterator;
use rayon::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::time::Instant;

mod tasks;

const W: usize = 104;

type Rule = [u8; W];
type RuleSet = Vec<Rule>;
type Path = Vec<usize>;
type IntMat = [[i64; W + 1]; W + 1];

fn line_to_rule(line: &str) -> Rule {
    let mut ret = [0; W];
    let mut i = 0;
    for ch in line.chars() {
        let c = match ch {
            '0' | '1' | '_' => Some(ch as u8),
            _ => None,
        };
        if let Some(x) = c {
            ret[i] = x;
            i += 1;
            if i == W - 1 {
                break;
            }
        }
    }
    ret
}

fn load_data(filename: &str) -> RuleSet {
    let content = fs::read_to_string(filename).expect("Failed to open file");
    let ret: RuleSet = content.lines().map(line_to_rule).collect();
    println!("Loaded {} rules", ret.len());
    ret
}

fn build_s_mat(rules: &RuleSet) -> (usize, IntMat) {
    println!("build_s_mat");
    let n = rules.len();
    let mut s_mat = [[0; W + 1]; W + 1];
    println!("W={}", W);
    s_mat.par_iter_mut().enumerate().for_each(|(i, row)| {
        (i + 1..=W).for_each(|j| {
            let w = (j - i) as i64;
            let valueset: HashSet<u64> = rules
                .iter()
                .map(|rule| {
                    let mut s = DefaultHasher::default();
                    Hash::hash_slice(&rule[i..j], &mut s);
                    s.finish()
                })
                .collect();
            let k = valueset.len() as i64;
            let b = (k as f64).log2().ceil() as i64;
            let saving = (w - b) * n as i64;
            let cost = if w > 20 { k * (w + b) } else { 0 };
            let net_saving = saving - cost;
            row[j] = net_saving;
        })
    });
    (n, s_mat)
}

fn find_optimal_cuts(s_mat: &IntMat) -> (Vec<i64>, Vec<Path>) {
    println!("find_optimal_cuts");
    let mut s = vec![0; W + 1];
    let mut path = Vec::with_capacity(W + 1) as Vec<Path>;
    for j in 0..=W {
        s[j] = s_mat[0][j];
        let mut best_i = None;
        for i in 0..j {
            let better = s[i] + s_mat[i][j];
            if s[j] < better {
                s[j] = better;
                best_i = Some(i);
            }
        }
        if let Some(i) = best_i {
            let pathj = [path[i].clone(), vec![i]].concat();
            path.push(pathj);
        } else {
            path.push(vec![]);
        }
    }
    (s, path)
}

fn get_savings(cuts: &Path, s_mat: &IntMat) -> i64 {
    let v = s_mat.len() - 1;
    let mut start = 0;
    let mut ret = 0;
    for &c in cuts {
        ret += s_mat[start][c];
        start = c;
    }
    ret += s_mat[start][v];
    ret
}

fn main() {
    let args = clap::App::new("Rustcuts")
        .arg(clap::Arg::with_name("input_file").takes_value(true))
        .get_matches();
    let input_file = args
        .value_of("input_file")
        .expect("Must provide an input file.");
    let input_path = format!("../neurocuts/classbench/{}", input_file);
    let output_path = format!("./data/{}_mat", input_file);
    tasks::binarize::run(&input_path, &output_path);
    // let rules = load_data(&input_path);
    // let now = Instant::now();
    // let (n, s_mat) = build_s_mat(&rules);
    // println!("Took {}ms.", now.elapsed().as_millis());
    // let (s, path) = find_optimal_cuts(&s_mat);
    // let total_cost = (W * n) as i64;
    // println!("Total uncompressed cost = {}", total_cost);
    // println!("Saving = {}", s[W]);
    // println!("Saving % = {}", s[W] as f64 / total_cost as f64);
    // println!("Cut points = {:?}", path[W]);
    // println!("Verify savings = {}", get_savings(&path[W], &s_mat));
}
