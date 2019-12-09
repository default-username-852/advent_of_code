mod computer;

use std::io::{stdin, BufRead};
use crate::computer::*;

fn main() {
    let stdin = stdin();
    
    let mut lines = stdin.lock().lines().map(|e| e.unwrap());
    
    let program: Vec<i64> = lines.next().unwrap().split(",").map(|e| e.parse::<i64>().unwrap()).collect();

    let c = Computer::new(program.clone());

    let out = c.run_blocking(&[2]);

    println!("{:?}", out);
}/*

fn gen_permutations(options: &mut Vec<i64>) -> Vec<Vec<i64>> {
    let mut c: Vec<i64> = iter::repeat(0).take(options.len()).collect();
    
    let mut out = Vec::new();
    
    out.push(options.clone());
    
    let mut i = 0;
    while i < options.len() {
        if c[i] < i as i64 {
            if i % 2 == 0 {
                options.swap(0, i);
            } else {
                options.swap(c[i] as usize, i);
            }
            out.push(options.clone());
            
            c[i] += 1;
            
            i = 0;
        } else {
            c[i] = 0;
            i += 1;
        }
    }
    
    out
}*/