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
use std::collections::hashmap::HashMap;

fn process_status_file(status_path: &Path) {
    let mut status_file = BufferedReader::new(File::open(status_path));
    let mut status_data = HashMap::new();
    for line in status_file.lines() {
        let linetext = match line {
            Err(why) => fail!("{}", why.desc),
            Ok(l) => l
        };
        let parts: Vec<&str> = linetext.as_slice().splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            status_data.insert(key.to_string(), value.to_string());
        };
    }

    let name_key = &("Name".to_string());
    let pid_key = &("Pid".to_string());
    let ppid_key = &("PPid".to_string());

    if status_data.contains_key(name_key) &&
        status_data.contains_key(pid_key) &&
        status_data.contains_key(ppid_key) {
            println!("{}#{} -> {}",
                     status_data.get(name_key),
                     status_data.get(pid_key),
                     status_data.get(ppid_key));
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
                process_status_file(&status_path);
            }
        }
    }
}

fn main() {
    dump_process_info();
}
