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

use std::path::Path;
use std::fs;
use std::io::prelude::*;
use std::fs::File;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

#[derive(Clone,Debug)]
struct ProcessRecord {
    name: String,
    pid: i32,
    ppid: i32,
}

#[derive(Clone,Debug)]
struct ProcessTreeNode {
    record: ProcessRecord,  // the node owns the associated record
    children: Vec<ProcessTreeNode>, // nodes own their children
}

#[derive(Clone,Debug)]
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
    let mut pid : Option<i32> = None;
    let mut ppid : Option<i32> = None;
    let mut name : Option<String> = None;

    let mut reader = std::io::BufReader::new(File::open(status_path).unwrap());
    loop {
        let mut linebuf = String::new();
        match reader.read_line(&mut linebuf) {
            Ok(_) => {
                if linebuf.is_empty() {
                    break;
                }
                let parts : Vec<&str> = linebuf[..].splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    match key {
                        "Name" => name = Some(value.to_string()),
                        "Pid" => pid = value.parse().ok(),
                        "PPid" => ppid = value.parse().ok(),
                        _ => (),
                    }
                }
            },
            Err(_) => break,
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
    let proc_directory = Path::new("/proc");

    // find potential process directories under /proc
    let proc_directory_contents = fs::read_dir(&proc_directory).unwrap();
    proc_directory_contents.filter_map(|entry| {
        let entry_path = entry.unwrap().path();
        if fs::metadata(entry_path.as_path()).unwrap().is_dir() {
            let status_path = entry_path.join("status");
            if let Ok(metadata) = fs::metadata(status_path.as_path()) {
                if metadata.is_file() {
                    return get_process_record(status_path.as_path());
                }
            }
        }
        None
    }).collect()
}

fn populate_node_helper(node: &mut ProcessTreeNode, pid_map: &HashMap<i32, &ProcessRecord>, ppid_map: &HashMap<i32, Vec<i32>>) {
    let pid = node.record.pid; // avoid binding node as immutable in closure
    let child_nodes = &mut node.children;
    match ppid_map.get(&pid) {
        Some(children) => {
            child_nodes.extend(children.iter().map(|child_pid| {
                let record = pid_map[child_pid];
                let mut child = ProcessTreeNode::new(record);
                populate_node_helper(&mut child, pid_map, ppid_map);
                child
            }));
        },
        None => {},
    }
}

fn populate_node(node : &mut ProcessTreeNode, records: &Vec<ProcessRecord>) {
    // O(n): build a mapping of pids to vectors of children.  That is, each
    // key is a pid and its value is a vector of the whose parent pid is the key
    let mut ppid_map : HashMap<i32, Vec<i32>> = HashMap::new();
    let mut pid_map : HashMap<i32, &ProcessRecord> = HashMap::new();
    for record in records.iter() {
        // entry returns either a vacant or occupied entry.  If vacant,
        // we insert a new vector with this records pid.  If occupied,
        // we push this record's pid onto the vec
        pid_map.insert(record.pid, record);
        match ppid_map.entry(record.ppid) {
            Vacant(entry) => { entry.insert(vec![record.pid]); },
            Occupied(mut entry) => { entry.get_mut().push(record.pid); },
        };
    }

    // With the data structures built, it is off to the races
    populate_node_helper(node, &pid_map, &ppid_map);
}

fn build_process_tree() -> ProcessTree {
    let records = get_process_records();
    let mut tree = ProcessTree {
        root : ProcessTreeNode::new(
            &ProcessRecord {
                name: "/".to_string(),
                pid: 0,
                ppid: -1
            })
    };

    // recursively populate all nodes in the tree starting from root (pid 0)
    {
        let root = &mut tree.root;
        populate_node(root, &records);
    }
    tree
}

fn print_node(node : &ProcessTreeNode, indent_level : i32) {
    // print indentation
    for _ in 0..indent_level {
        print!("  ");
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
