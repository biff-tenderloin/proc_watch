use proc_maps::{get_process_maps, MapRange, Pid};
use std::process;
use std::collections::HashSet;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut pid = process::id();
    if args.len() > 1 {
        pid = *&args[1].parse::<u32>().unwrap();
    }

    let mut libs = HashSet::new();

    println!("My pid is {}", pid);
    let maps = get_process_maps(pid as Pid).unwrap();
    for map in maps {
        if map.filename().is_some() {
            let p = map.filename().unwrap();
            if p.is_file() && p.extension().is_some() {
                if p.extension().unwrap() == "so" {
                    //println!("{}", p.display());
                    let path_buf = p.to_path_buf();
                    if !libs.contains(&path_buf) {
                        libs.insert(path_buf);
                    }
                }
            }
        }
    }

    //println!("{} libraries", libs.len());
    for p in libs {
        println!("{}", p.display());
    }
}
