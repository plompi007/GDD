//! `EnergyGraph`: the topological (non-physical) half of docs/GDD.md's tick
//! order (§2.4 step 1) — "electricity → light/heat/rotary". Mirrors §5.6's
//! sketch: nodes are keyed by the placed-part `id` string (not `Entity`),
//! so wiring stays readable in level JSON and stable across resets.
//!
//! Only `ELECTRIC` exists as a signal today because it's the only energy
//! type any M4 part actually produces/consumes through this graph — ROTARY
//! is handled by `gear_train.rs`'s own proximity/belt coupling instead,
//! since gear meshing is geometric, not topological. Add variants to
//! [`EnergySignal`] as later milestones add parts that need them (LIGHT for
//! `cell_solar`, THERMAL for `fuse_cord`, etc.) — don't pre-build unused
//! ones now.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::sim::SimSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnergySignal {
    Electric(bool),
}

/// docs/GDD.md §5.6's `EnergyGraph` resource. `edges` (the wiring, built
/// once from a level's `connections: [{kind: WIRE}]` at load time) and
/// `signals` (this tick's values, re-emitted by source systems every tick —
/// see CLAUDE.md rule 2, nothing here is allowed to persist state that
/// isn't rebuilt on `Running`) are kept separate so `clear()` on reset can
/// drop both without the two ever getting out of sync.
#[derive(Resource, Default)]
pub struct EnergyGraph {
    signals: BTreeMap<(String, String), EnergySignal>,
    edges: BTreeMap<(String, String), Vec<(String, String)>>,
}

impl EnergyGraph {
    pub fn connect(&mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) {
        self.edges
            .entry((from_node.to_string(), from_port.to_string()))
            .or_default()
            .push((to_node.to_string(), to_port.to_string()));
    }

    pub fn emit(&mut self, node_id: &str, port_id: &str, signal: EnergySignal) {
        self.signals
            .insert((node_id.to_string(), port_id.to_string()), signal);
    }

    pub fn read(&self, node_id: &str, port_id: &str) -> Option<EnergySignal> {
        self.signals
            .get(&(node_id.to_string(), port_id.to_string()))
            .copied()
    }

    pub fn clear(&mut self) {
        self.signals.clear();
        self.edges.clear();
    }

    /// Pushes every emitted out-port signal along its wired edges into the
    /// matching in-port. Source parts (e.g. `outlet_power`) re-`emit()`
    /// their own value earlier in [`SimSet::EnergyPropagate`] every tick, so
    /// two relaxation passes here are enough to settle any chain up to
    /// depth 2 (source → relay → consumer, e.g. outlet → switch → motor) —
    /// docs/GDD.md §5.6's "cycles resolved in 2 passes max".
    pub fn propagate(&mut self) {
        for _ in 0..2 {
            let mut updates = Vec::new();
            for ((from_node, from_port), targets) in &self.edges {
                if let Some(signal) = self.signals.get(&(from_node.clone(), from_port.clone())) {
                    for to in targets {
                        updates.push((to.clone(), *signal));
                    }
                }
            }
            for (key, signal) in updates {
                self.signals.insert(key, signal);
            }
        }
    }
}

fn propagate_energy_graph(mut graph: ResMut<EnergyGraph>) {
    graph.propagate();
}

/// A purely cosmetic record of which two live entities a `WIRE` connection
/// joins — unlike [`EnergyGraph`] itself (keyed by part-id strings, the
/// real electrical logic), this exists only so `render_fx.rs` has
/// `Transform`s to draw a line between. Nothing in this module or
/// `sim::SimSet::EnergyPropagate` reads it; `level_load.rs` and
/// `input.rs`'s connect-tool spawn one alongside every real
/// `EnergyGraph::connect()` call, mirroring exactly how
/// `rope_network::RopeConnection`/`gear_train::BeltConnection` already
/// pair a physics-driving component with the entities its own line
/// should connect.
#[derive(Component)]
pub struct WireConnection {
    pub from: Entity,
    pub to: Entity,
}

/// Registers [`EnergyGraph`] and wires its propagation step into
/// `SimSet::EnergyPropagate` (docs/GDD.md §2.4 step 1). Source systems
/// (`outlet_power`'s, etc.) should also live in `SimSet::EnergyPropagate`;
/// exact ordering against this system doesn't matter because `signals`
/// persists tick-to-tick (only `clear()` on reset drops it) — a source
/// whose own emit happens to run after this propagate just takes one extra
/// tick (1/120s) to reach a downstream consumer, which is negligible next
/// to `auto_start_once`'s 1-second startup delay.
pub struct EnergyGraphPlugin;

impl Plugin for EnergyGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyGraph>().add_systems(
            FixedUpdate,
            propagate_energy_graph.in_set(SimSet::EnergyPropagate),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagates_a_two_hop_chain_in_one_call() {
        let mut graph = EnergyGraph::default();
        graph.connect("outlet", "out", "switch", "in");
        graph.connect("switch", "out", "motor", "in");
        graph.emit("outlet", "out", EnergySignal::Electric(true));
        // "switch" hasn't re-emitted yet (that's its own system's job) —
        // simulate the source-only tick 1, then a relay tick 2's emit.
        graph.propagate();
        assert_eq!(
            graph.read("switch", "in"),
            Some(EnergySignal::Electric(true))
        );
        graph.emit("switch", "out", EnergySignal::Electric(true));
        graph.propagate();
        assert_eq!(
            graph.read("motor", "in"),
            Some(EnergySignal::Electric(true))
        );
    }

    #[test]
    fn clear_drops_both_edges_and_signals() {
        let mut graph = EnergyGraph::default();
        graph.connect("a", "out", "b", "in");
        graph.emit("a", "out", EnergySignal::Electric(true));
        graph.clear();
        graph.propagate();
        assert_eq!(graph.read("b", "in"), None);
    }
}
