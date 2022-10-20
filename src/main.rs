use proc_maps::{get_process_maps, Pid};
use std::{process, thread};
use std::process::{Command, Stdio};
use std::collections::HashSet;
use std::path::PathBuf;
use structopt::StructOpt;
use termion::{color, style};
use sysinfo::{Pid as sysPid, System, SystemExt};
use std::sync::{Arc, Mutex};
use std::env;
use std::path::Path;
use std::ffi::OsStr;

#[derive(StructOpt, Debug)]
#[structopt(name = "process")]
enum CommandLine {
    #[structopt(help="Watch process given by pid")]
    Pid {
        #[structopt(short, long)]
        pid: Pid,

        #[structopt(short, long)]
        debug: bool,

        /// Number of seconds for polling
        #[structopt(short = "m", long)]
        milliseconds: Option<u64>,
    },

    #[structopt(help="Start watched process given by command")]
    Start {
        #[structopt(short, long)]
        command: String,

        #[structopt(short, long)]
        debug: bool,

        /// Number of seconds for polling
        #[structopt(short = "m", long)]
        milliseconds: Option<u64>,

        #[structopt(parse(from_str))]
        external_args: Vec<String>,
    },

    #[structopt(help="Watch self")]
    Me {
        #[structopt(short, long)]
        debug: bool,
    },
}


// proc_watch pid 12345 -o output.txt -s 5
// proc_watch start ./test-app -o test-app.log

fn main() {

    let args: CommandLine = CommandLine::from_args();
    match args {
        CommandLine::Pid {pid, debug, milliseconds } => {
            if debug {
                println!("Watching pid {}", pid);
                println!("debug: {}", debug);
                println!("milliseconds: {:?}", milliseconds);
            }
            if pid < 1 {
                println!("Invalid pid");
                process::exit(1);
            }
            print_report(watch(pid, milliseconds.unwrap_or(0)));
        },
        CommandLine::Start {
            command, debug, milliseconds, external_args } => {
            if debug {
                println!("command: {}", command);
                println!("debug: {}", debug);
                println!("milliseconds: {:?}", milliseconds);
                println!("external_args: {:?}", external_args);
            }
            start_and_watch(command, external_args, milliseconds.unwrap_or(0));
        },
        CommandLine::Me { debug} => {
            if debug {
                println!("debug: {}", debug);
            }
            let pid = process::id();
            println!("Watching current process: {}", pid);
            print_report(watch(pid as Pid, 0));
        },
    }
}


fn prog() -> Option<String> {
    env::current_exe().ok()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}

fn print_report(contents: HashSet<PathBuf>) {
    println!("{}{}===================================================", color::Fg(color::Green), style::Bold);
    println!("{} report:", prog().unwrap());
    println!("{}{}---------------------------------------------------", color::Fg(color::Green), style::Bold);
    print!("{}", style::Reset);
    for content in contents {
        println!("{}{}", color::Fg(color::LightWhite), content.display());
    }
    println!("{}{}---------------------------------------------------", color::Fg(color::Green), style::Bold);
    print!("{}{}", color::Fg(color::Reset), style::Reset);
}

fn print_msg(msg: String) {
    println!("{}{}{}{}{}",
             color::Fg(color::Blue), style::Italic,
             msg,
             color::Fg(color::Reset), style::Reset);
}


fn watch(pid: Pid, poll_in_milliseconds: u64) -> HashSet<PathBuf> {
    let mut libs = HashSet::new();
    let simple_pid = pid as i32;
    let s = System::new_all();

    let mut done = false;
    while done == false {
        match s.process(sysPid::from(simple_pid)) {
            None => done = true,
            Some(_) => {
                match get_process_maps(pid) {
                    Ok(maps) =>
                        {
                            for map in maps {
                                if map.filename().is_some() {
                                    let p = map.filename().unwrap();
                                    if p.is_file() && p.extension().is_some() {
                                        if p.extension().unwrap() == "so" {
                                            let path_buf = p.to_path_buf();
                                            if !libs.contains(&path_buf) {
                                                libs.insert(path_buf);
                                            }
                                        }
                                    }
                                }
                            }

                            if poll_in_milliseconds > 0 {
                                thread::sleep(std::time::Duration::from_millis(poll_in_milliseconds as u64));
                                //println!("Polling...");
                            } else {
                                done = true;
                            }
                        },
                    Err(_)     =>
                        done = true,
                }

            }
        }
    }

    libs
}

fn start_and_watch(command: String, external_args: Vec<String>, poll_in_milliseconds: u64) {
    let mut child = Command::new(command)
        .args(external_args)
        .stdout(Stdio::inherit())
        .spawn()
        .expect("failed to execute process");

    // if poll_in_milliseconds > 0, the will never exit because we are polling a process
    // that never exits because we are watching it and never get to the wait() call
    //
    // need to thread this part
    //let report = watch(child.id() as Pid, poll_in_milliseconds);

    let child_id = child.id();
    let report = Arc::new(Mutex::new(HashSet::new()));
    let report_clone = report.clone();
    let handle = thread::spawn(move || {
        let r = watch(child_id as Pid, poll_in_milliseconds);
        let mut rpt = report_clone.lock().unwrap();
        for item in r {
            if !rpt.contains(&item) {
                //let t = item.clone();
                rpt.insert(item);
                //println!("{}{}", color::Fg(color::LightWhite), t.display());
            }
        }
    });

    let child_handle = thread::spawn(move || {
        let this_prog = prog().unwrap();

        // Wait for the process to exit.
        match child.wait() {
            Ok(status) =>
                print_msg(format!("[{}] Finished, status of {}", this_prog, status)),
            Err(e)     =>
                println!("[{}] Failed, error: {}", this_prog, e)
        }
    });

    handle.join().unwrap();
    child_handle.join().unwrap();

    print_report(report.lock().unwrap().clone());
}
