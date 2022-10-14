use proc_maps::{get_process_maps, Pid};
use std::process;
use std::process::Command;
use std::collections::HashSet;
//use std::env;
use std::path::PathBuf;
use structopt::StructOpt;
use std::io::{stdin, stdout, Write};

#[derive(StructOpt, Debug)]
#[structopt(name = "process")]
enum CommandLine {
    #[structopt(help="Watch process given by pid")]
    Pid {
        #[structopt(short, long)]
        pid: Pid,

        #[structopt(short, long)]
        debug: bool,

        /// Output file
        #[structopt(short, long, parse(from_os_str))]
        output: Option<PathBuf>,

        /// Number of seconds for polling
        #[structopt(short = "s", long)]
        seconds: Option<u32>,
    },

    #[structopt(help="Start watched process given by command")]
    Start {
        #[structopt(short, long)]
        command: String,

        #[structopt(short, long)]
        debug: bool,

        /// Output file
        #[structopt(short, long, parse(from_os_str))]
        output: Option<PathBuf>,

        /// Number of seconds for polling
        #[structopt(short = "s", long)]
        seconds: Option<u32>,
    },

    #[structopt(help="Watch self")]
    Me {
        #[structopt(short, long)]
        debug: bool,

        /// Output file
        #[structopt(short, long, parse(from_os_str))]
        output: Option<PathBuf>,

        /// Number of seconds for polling
        #[structopt(short = "s", long)]
        seconds: Option<u32>,
    },
}


// proc_watch pid 12345 -o output.txt -s 5
// proc_watch start ./test-app -o test-app.log

fn main() {

    let args: CommandLine = CommandLine::from_args();
    match args {
        CommandLine::Pid { pid, debug, output, seconds } => {
            if debug {
                println!("Watching pid {}", pid);
                println!("debug: {}", debug);
                println!("output: {:?}", output);
                println!("seconds: {:?}", seconds);
            }
            if pid < 1 {
                println!("Invalid pid");
                process::exit(1);
            }
            watch(pid);
        },
        CommandLine::Start { command, debug, output, seconds } => {
            if debug {
                println!("command: {}", command);
                println!("debug: {}", debug);
                println!("output: {:?}", output);
                println!("seconds: {:?}", seconds);
            }
            start_and_watch(command);
        },
        CommandLine::Me { debug, output, seconds } => {
            if debug {
                println!("debug: {}", debug);
                println!("output: {:?}", output);
                println!("seconds: {:?}", seconds);
            }
            let pid = process::id();
            println!("Watching current process: {}", pid);
            watch(pid as Pid);
        },
    }
}

fn watch(pid: Pid) {
    let mut libs = HashSet::new();

    // TODO: execute the following in a loop to exit when the process exits or terminated by user
    // TODO: use a thread to execute the following in a loop
    //       while the main thread waits for user input to terminate the process
    let maps = get_process_maps(pid).unwrap();
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

fn start_and_watch(command: String) {
    let mut cmd = Command::new(command);

    let mut child = cmd.spawn()
        .expect("failed to execute process");
    watch(child.id() as Pid);

    let output = cmd.output();

    // Wait for the process to exit.
    match child.wait() {
        Ok(status) => println!("Finished, status of {}", status),
        Err(e)     => println!("Failed, error: {}", e)
    }

    if let Ok(output) = output {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

}
