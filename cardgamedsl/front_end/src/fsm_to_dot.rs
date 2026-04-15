/*
    We give an extra output for checking if the game generated is visually fine.
    This file converts the IR into both DOT and SVG formats.
*/

use layout::core::color::Color;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::ir::*;

// Layout crate imports for pure-rust SVG generation
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::topo::layout::VisualGraph;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};

pub fn fsm_to_svg<Ctx: AstContext>(
    fsm: &Ir<Payload<Ctx>>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>>
where
    Ctx: Serialize + DeserializeOwned,
{
    // 1. Create visual graph
    let mut vg = VisualGraph::new(Orientation::TopToBottom);

    // 2. Node style
    let style = StyleAttr::new(
        Color::new(0x333333),       // dark grey stroke
        2,                          // stroke width
        Some(Color::new(0xF9F9F9)), // light grey fill
        5,                          // rounded corners
        12,                         // font size
    );

    let orientation = Orientation::TopToBottom;
    let node_size = Point::new(100.0, 50.0);

    // 3. Map for node handles
    let mut nodes = std::collections::HashMap::new();

    // 4. Entry node
    let entry_element = Element::create(
        ShapeKind::Circle("entry".to_string()),
        style.clone(),
        orientation,
        Point::new(20.0, 20.0),
    );
    let entry_handle = vg.add_node(entry_element);

    // 5. States
    for state_id in fsm.states.keys() {
        let label = format!("id=\"State: {}\"", state_id.raw());
        let block_label = format!("State: {}", state_id.raw());

        let state_element = Element::create_with_properties(
            ShapeKind::Box(block_label), // displayed text
            style.clone(),
            orientation,
            node_size,
            label, // text inside box
        );

        // If your library allows setting ID, uncomment this:
        // state_element.set_id(node_id);

        let handle = vg.add_node(state_element);
        nodes.insert(state_id.raw(), handle);
    }

    // 6. Connect entry to start state
    if let Some(&start_handle) = nodes.get(&fsm.entry.raw()) {
        vg.add_edge(Arrow::default(), entry_handle, start_handle);
    }

    // 7. Transitions
    for (state_id, edges) in &fsm.states {
        let &source_handle = nodes.get(&state_id.raw()).unwrap();
        for edge in edges.iter() {
            if let Some(&target_handle) = nodes.get(&edge.to.raw()) {
                let mut arrow = Arrow::default();
                arrow.text = edge.payload.to_string();
                vg.add_edge(arrow, source_handle, target_handle);
            }
        }
    }

    // 8. Render SVG
    let mut svg_writer = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg_writer); // enable layout fitting

    let content = svg_writer.finalize();

    // 9. Write to file
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;

    Ok(())
}

/// Generates a standard .dot file. 
/// Useful if the user HAS Graphviz or for use in the VS Code Webview.
pub fn fsm_to_dot<Ctx: AstContext>(
    fsm: &Ir<Payload<Ctx>>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>>
where
    Ctx: Serialize + DeserializeOwned,
{
    let mut file = File::create(path)?;

    writeln!(file, "digraph CFG {{")?;
    writeln!(file, "  graph [splines=ortho, nodesep=1.0, ranksep=1.0, concentrate=true];")?;
    writeln!(file, "  node [shape=box, fontname=\"Arial\", style=filled, fillcolor=\"#ffffff\", color=\"#333333\"];")?;
    writeln!(file, "  edge [fontname=\"Arial\", fontsize=9, arrowsize=0.8];")?;

    writeln!(file, "  entry [shape=point];")?;
    writeln!(file, "  entry -> {:?};", fsm.entry.raw())?;

    for (state_id, edges) in &fsm.states {
        for edge in edges.iter() {
            let label = edge.payload.to_string().replace('"', "\\\"");
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