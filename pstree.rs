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

#[deriving(Clone,Show)]
struct ProcessRecord {
    name: String,
    pid: int,
    ppid: int
}

#[deriving(Clone,Show)]
struct ProcessTreeNode {
    record: ProcessRecord,  // the node owns the associated record
    children: Vec<ProcessTreeNode>, // nodes own their children
}

#[deriving(Clone,Show)]
struct ProcessTree {
    root: ProcessTreeNode, // tree owns ref to root node
}

impl ProcessTreeNode {
    // constructor
    fn new(record : &ProcessRecord) -> ProcessTreeNode {
        ProcessTreeNode { record: (*record).clone(), children: Vec::new() }
    }
}


// Given a status file path, return a hashmap with the following form:
// pid -> ProcessRecord
fn get_process_record(status_path: &Path) -> Option<ProcessRecord> {
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


// build a simple struct (ProcessRecord) for each process
fn get_process_records() -> Vec<ProcessRecord> {
    let mut records : Vec<ProcessRecord> = Vec::new();
    let proc_directory = Path::new("/proc");

    // find potential process directories under /proc
    let proc_directory_contents = match fs::readdir(&proc_directory) {
        Err(why) => fail!("{}", why.desc),
        Ok(res) => res
    };

    for entry in proc_directory_contents.iter().filter(|entry| entry.is_dir()) {
        let status_path = entry.join("status");
        if status_path.exists() {
            match get_process_record(&status_path) {
                Some(record) => records.push(record),
                None => (),
            }
        }
    }
    records
}

fn populate_node(node : &mut ProcessTreeNode, records: &Vec<ProcessRecord>) {
    // populate the node by finding its children... recursively
    let pid = node.record.pid; // avoid binding node as immutable in closure
    for record in records.iter().filter(|record| record.ppid == pid) {
        let mut child = ProcessTreeNode::new(record);
        populate_node(&mut child, records);
        node.children.push(child);
    }
}

fn build_process_tree() -> ProcessTree {
    let records = get_process_records();
    let mut tree = ProcessTree {
        root : ProcessTreeNode::new(
            &ProcessRecord { name: "/".to_string(), pid: 0, ppid: -1 })
    };

    // recursively populate all nodes in the tree starting from root (pid 0)
    {
        let root = &mut tree.root;
        populate_node(root, &records);
    }
    tree
}

fn print_node(node : &ProcessTreeNode, indent_level : int) {
    // print indentation
    for _ in range(0, indent_level * 2) {
        print!(" ");
    }
    println!("- {} #{}", node.record.name, node.record.pid);
    for child in node.children.iter() {
        print_node(child, indent_level + 1);  // recurse
    }
}

fn main() {
    let ptree = build_process_tree();
    print_node(&(ptree.root), 0)
}
