use front_end::ir::{Edge, Ir};

use crate::engine_payload::EnginePayload;

pub type EngineIr = Ir<EnginePayload>;

pub fn convert_ir(ir: front_end::ir::Ir<front_end::ir::LoweredPayLoad>) -> EngineIr {
    let states = ir
        .states
        .into_iter()
        .map(|(state_id, edges)| {
            let converted_edges: Vec<Edge<EnginePayload>> = edges
                .into_iter()
                .map(|edge| Edge {
                    to: edge.to,
                    payload: EnginePayload::from(edge.payload),
                    meta: edge.meta,
                })
                .collect();
            (state_id, converted_edges)
        })
        .collect();

    Ir {
        states,
        entry: ir.entry,
        goal: ir.goal,
    }
}