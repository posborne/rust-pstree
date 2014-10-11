// A version of pstree targetting linux written in rust!
//
// This is based on the following exercise from the excellent
// book "The Linux Programming Interface" by Michael Kerridsk.
//
//----------------------------------------------------------------------
//
// Write a program that draws a tree showing the hierarchical
// parent-child relationships of all processes on the system, going all
// the way back to init.  For each process, the program should display
// the process ID and the command being executed.  The output of the
// program should be similar to that produced by pstree(1), although it
// does need not to be as sophisticated.  The parent of each process on
// the system can be found by inspecing the PPid: line of all of the
// /proc/PID/status files on the system.  Be careful to handle the
// possibilty that a process's parent (and thus its /proc/PID directory)
// disappears during the scan of all /proc/PID directories.

// Implementation Notes
// --------------------
// The linux /proc filesystem is a virtual filesystem that provides information
// about processes running on a linux system among other things.  The /proc
// filesystem contains a directory, /proc/<pid>, for each running process in
// the system.
//
// Each process directory has a status file with contents including a bunch
// of different items, notably the process name and its parent process id (ppid).
// And with that information, we can build the process tree.

use std::io::fs::PathExtensions;
use std::io::fs;
use std::io::File;
use std::io::BufferedReader;
use std::fmt;

struct ProcessRecord {
    name: String,
    pid: int,
    ppid: int
}

impl fmt::Show for ProcessRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ProcessRecord [ name: {}, pid: {}, ppid: {} )",
               self.name,
               self.pid,
               self.ppid)
    }
}

// Given a status file path, return a hashmap with the following form:
// pid -> ProcessRecord
fn get_status_info(status_path: &Path) -> Option<ProcessRecord> {
    let mut pid : Option<int> = None;
    let mut ppid : Option<int> = None;
    let mut name : Option<String> = None;

    let mut status_file = BufferedReader::new(File::open(status_path));
    for line in status_file.lines() {
        let unwrapped = line.unwrap(); // need a new lifeline
        let parts : Vec<&str> = unwrapped.as_slice().splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            match key {
                "Name" => name = Some(value.to_string()),
                "Pid" => pid = from_str(value),
                "PPid" => ppid = from_str(value),
                _ => (),
            }
        }
    }

    return if pid.is_some() && ppid.is_some() && name.is_some() {
        Some(ProcessRecord { name: name.unwrap(), pid: pid.unwrap(), ppid: ppid.unwrap() })
    } else {
        None
    }
}


fn dump_process_info() {
    let proc_directory = Path::new("/proc");
    let proc_directory_contents = match fs::readdir(&proc_directory) {
        Err(why) => fail!("{}", why.desc),
        Ok(res) => res
    };
    for entry in proc_directory_contents.iter() {
        if entry.is_dir() {
            let status_path = entry.join("status");
            if status_path.exists() {
                let record = get_status_info(&status_path);
                match record {
                    Some(record) => println!("{}", record),
                    None => ()
                }
            }
        }
    }
}

fn main() {
    dump_process_info();
}
