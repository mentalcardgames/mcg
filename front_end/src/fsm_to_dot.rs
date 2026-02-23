/*
    We give an extra output for checking if the game generated is visually fine.
    In this file is defined how to convert the IR into a visual depiction of it.
*/

use std::fs::File;
use std::io::Write;
use std::path::Path;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::ir::*;

pub fn fsm_to_dot<Ctx: AstContext>(fsm: &Ir<Payload<Ctx>>, path: &Path) -> Result<(), Box<dyn std::error::Error>>
    where
        Ctx: Serialize + DeserializeOwned
{
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
        // writeln!(file, "  {:?} [xlabel=\"Block: {:?}\"];", state_id.raw(), state_id.raw())?;
        
        for edge in edges.iter() {
            let label = String::from(edge.payload.to_string()).replace('"', "\\\"");
            writeln!(
                file, 
                "  {:?} -> {:?} [xlabel=\" {} \"];", 
                state_id.raw(), 
                edge.to.raw(), 
                label
            )?;
        }
    }

    writeln!(file, "}}")?;
    Ok(())
}