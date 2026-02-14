use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::ir::*;

pub fn fsm_to_dot(fsm: &Ir<PayloadT>, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path).unwrap();
    writeln!(file, "digraph FSM {{").unwrap();
    writeln!(file, "  rankdir=LR;").unwrap();
    writeln!(file, "  node [shape = circle];").unwrap();

    for (state_id, edges) in &fsm.states {
        for edge in edges.iter() {
            writeln!(file, "  {:?} -> {:?} [label=\"{}\"];", state_id.raw(), edge.to.raw(), &edge.payload.raw()).unwrap();
        }
    }
    

    // mark start and end
    writeln!(file, "  start [shape=point];").unwrap();
    writeln!(file, "  start -> {:?};", fsm.entry.raw()).unwrap();
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