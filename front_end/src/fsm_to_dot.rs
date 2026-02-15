use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::ir::*;

pub fn fsm_to_dot(fsm: &Ir<PayloadT>, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    
    writeln!(file, "digraph CFG {{")?;
    // Layout Engine Tuning
    writeln!(file, "  graph [splines=ortho, nodesep=1.0, ranksep=1.0, concentrate=true];")?;
    writeln!(file, "  node [shape=box, fontname=\"Consolas, 'Courier New', monospace\", style=filled, fillcolor=\"#ffffff\", bordercolor=\"#333333\"];")?;
    writeln!(file, "  edge [fontname=\"Consolas\", fontsize=9, arrowsize=0.8];")?;

    // Entry point
    writeln!(file, "  entry [shape=point];")?;
    writeln!(file, "  entry -> {:?};", fsm.entry.raw())?;

    for (state_id, edges) in &fsm.states {
        // Render each state as a clean rectangle
        writeln!(file, "  {:?} [label=\"Block: {:?}\"];", state_id.raw(), state_id.raw())?;
        
        for edge in edges.iter() {
            let label = edge.payload.raw().replace('"', "\\\"");
            writeln!(
                file, 
                "  {:?} -> {:?} [label=\" {} \"];", 
                state_id.raw(), 
                edge.to.raw(), 
                label
            )?;
        }
    }

    writeln!(file, "}}")?;
    Ok(())
}