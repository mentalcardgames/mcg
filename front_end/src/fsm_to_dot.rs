use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::fsm::*;

pub fn fsm_to_dot(fsm: &FSM, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path).unwrap();
    writeln!(file, "digraph FSM {{").unwrap();
    writeln!(file, "  rankdir=LR;").unwrap();
    writeln!(file, "  node [shape = circle];").unwrap();

    for (tid, trans) in &fsm.transitions {
        for (state_id, edges) in &fsm.states {
            for (edge_tid, to) in edges {
                if *edge_tid == *tid {
                    let label = format!("{:?}", trans);
                    writeln!(file, "  {} -> {} [label=\"{}\"];", state_id, to, label).unwrap();
                }
            }
        }
    }

    // mark start and end
    writeln!(file, "  start [shape=point];").unwrap();
    writeln!(file, "  start -> {};", fsm.entry).unwrap();
    // writeln!(file, "  {} [shape=doublecircle];", fsm.end).unwrap();

    writeln!(file, "}}").unwrap();

    clean_dot_file(path)
}

use regex::Regex;
use std::fs;

/// Reads a DOT file, removes internal quotes inside label="...",
/// and writes the cleaned result to another file.
pub fn clean_dot_file(
    path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    // Read file
    let data = fs::read_to_string(path)?;

    // Regex to capture label=" ... "
    let re = Regex::new(r#"label="(.*?)"\];"#)?;

    // Clean all labels
    let cleaned = re.replace_all(&data, |caps: &regex::Captures| {
        let inner = &caps[1];
        let inner_clean = inner.replace('"', "");
        format!("label=\"{}\"];", inner_clean)
    });

    // Write output
    let mut out = fs::File::create(path)?;
    out.write_all(cleaned.as_bytes())?;

    Ok(())
}