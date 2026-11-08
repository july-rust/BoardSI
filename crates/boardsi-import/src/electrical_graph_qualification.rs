//! D_056_03: deterministic, evidence-scoped electrical *design* graph qualification.
//!
//! D_056_02 imports source-local positive-area contact candidates. This module
//! checks their identities, constructs connected components and graph witnesses,
//! and accepts cross-layer edges ONLY with separately supplied process evidence.
//! Neither a nominal drill span nor parser `plated=true` nor a matching Net
//! constitutes such evidence. No result proves manufactured PCB continuity or SI.
use crate::layered_source_graph::{ContactKind, CopperFootprint, DeclaredNetCheck, LayeredSourceGraph};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Source coverage is not inferred from the parser's successful return. A
/// separate auditor must substantiate a complete conductive inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceCoverage {
    Partial,
    QualifiedSourceScope {
        sha256: [u8; 32],
        evidence_reference: String,
    },
}

/// Externally reviewed *design/process contract*, not manufactured-board proof.
/// A declared plated flag, mere drill span, or GUI metadata cannot create this.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarrelProcessWitness {
    pub sha256: [u8; 32],
    pub via_key: String,
    pub first_layer: u32,
    pub last_layer: u32,
    pub documented_process_reference: String,
    pub qualified_transition_reference: String,
}

/// Expected endpoints must originate in a separately established netlist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredNetPath {
    pub net: String,
    pub endpoint_source_keys: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualifiedEdgeKind {
    SourceLocalPositiveArea,
    EvidenceQualifiedBarrel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifiedDesignEdge {
    pub first: usize,
    pub second: usize,
    pub kind: QualifiedEdgeKind,
    /// Exact source keys or process-document reference, not renderer inference.
    pub evidence_reference: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenDisposition {
    /// Disconnect within an independently qualified *source* inventory.
    QualifiedSourceScope,
    /// The source graph is partial and may be missing an alternate connection.
    CandidateOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenWitness {
    pub net: String,
    pub endpoints: Vec<usize>,
    pub component_ids: Vec<usize>,
    pub disposition: OpenDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceNetConflict {
    pub component_id: usize,
    pub first_node: usize,
    pub second_node: usize,
    pub first_net: String,
    pub second_net: String,
    /// Positive-area / process-qualified graph path, not a manufactured short.
    pub path_nodes: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualificationReport {
    pub source_sha256: [u8; 32],
    pub source_scope_qualified: bool,
    pub edges: Vec<QualifiedDesignEdge>,
    pub components: Vec<Vec<usize>>,
    pub component_by_node: Vec<usize>,
    pub net_conflicts: Vec<SourceNetConflict>,
    pub open_witnesses: Vec<OpenWitness>,
    pub bridge_edges: Vec<(usize, usize)>,
    pub articulation_nodes: Vec<usize>,
    pub unverified_barrel_keys: Vec<String>,
    pub diagnostics: Vec<String>,
}

/// Perform source-scoped design graph analysis. The caller supplies the SHA
/// of the exact source bytes it is currently inspecting (e.g. re-import).
/// Do not feed any of these edges into manufactured/SI acceptance.
pub fn qualify(
    graph: &LayeredSourceGraph,
    current_source_sha256: [u8; 32],
    coverage: &SourceCoverage,
    barrel_proofs: &[BarrelProcessWitness],
    expected_paths: &[RequiredNetPath],
) -> Result<QualificationReport, &'static str> {
    if graph.source_sha256 != current_source_sha256 {
        return Err("D05603_STALE_SOURCE_SHA");
    }
    if !(2..=64).contains(&graph.declared_copper_layers) {
        return Err("D05603_INVALID_LAYER_COUNT");
    }
    let source_scope_qualified = match coverage {
        SourceCoverage::Partial => false,
        SourceCoverage::QualifiedSourceScope {
            sha256,
            evidence_reference,
        } => {
            if *sha256 != current_source_sha256 || evidence_reference.trim().is_empty() {
                return Err("D05603_INVALID_COVERAGE_EVIDENCE");
            }
            true
        }
    };
    // This input is deliberately a local-candidate graph. Refuse silently
    // pre-promoted edges and process flags from a legacy/importer shortcut.
    if !graph.verified_electrical_edges.is_empty() {
        return Err("D05603_UNTRUSTED_PREVERIFIED_EDGES");
    }
    if graph.nodes.len() > 50_000 || graph.geometry_contacts.len() > 100_000 {
        return Err("D05603_GRAPH_BOUNDS");
    }
    let n = graph.nodes.len();
    let mut keys = BTreeMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        if !(1..=graph.declared_copper_layers).contains(&node.physical_copper_layer)
            || node.source_key.trim().is_empty()
        {
            return Err("D05603_INVALID_NODE");
        }
        if keys.insert(node.source_key.as_str(), i).is_some() {
            return Err("D05603_DUPLICATE_SOURCE_KEY");
        }
    }

    let mut edges = Vec::new();
    let mut seen = BTreeSet::new();
    for hit in &graph.geometry_contacts {
        let a = graph
            .nodes
            .get(hit.first_node)
            .ok_or("D05603_MISSING_LOCAL_NODE")?;
        let b = graph
            .nodes
            .get(hit.second_node)
            .ok_or("D05603_MISSING_LOCAL_NODE")?;
        if hit.first_node == hit.second_node
            || a.physical_copper_layer != b.physical_copper_layer
            || a.physical_copper_layer != hit.layer
        {
            return Err("D05603_INVALID_LOCAL_CONTACT");
        }
        let expected_relation = match (&a.declared_net, &b.declared_net) {
            (Some(x), Some(y)) if x == y => DeclaredNetCheck::Same,
            (Some(_), Some(_)) => DeclaredNetCheck::Different,
            _ => DeclaredNetCheck::Unknown,
        };
        if hit.declared_net_check != expected_relation {
            return Err("D05603_LOCAL_NET_EVIDENCE_CONFLICT");
        }
        // Never relabel a ViaPad/ViaPad or SMD/SMD pair as a trace contact.
        let type_matches = matches!(
            (hit.kind, &a.footprint, &b.footprint),
            (ContactKind::TraceTrace, CopperFootprint::Stroke { .. }, CopperFootprint::Stroke { .. })
                | (ContactKind::TraceViaPad, CopperFootprint::Stroke { .. }, CopperFootprint::ViaPad { .. })
                | (ContactKind::TraceViaPad, CopperFootprint::ViaPad { .. }, CopperFootprint::Stroke { .. })
                | (ContactKind::TraceSmdPad, CopperFootprint::Stroke { .. }, CopperFootprint::SmdPad { .. })
                | (ContactKind::TraceSmdPad, CopperFootprint::SmdPad { .. }, CopperFootprint::Stroke { .. })
        );
        if !type_matches {
            return Err("D05603_CONTACT_KIND_MISMATCH");
        }
        let (first, second) = ordered(hit.first_node, hit.second_node);
        if seen.insert((first, second)) {
            edges.push(QualifiedDesignEdge {
                first,
                second,
                kind: QualifiedEdgeKind::SourceLocalPositiveArea,
                evidence_reference: format!(
                    "SOURCE_LOCAL:L{}:{}:{}",
                    hit.layer, a.source_key, b.source_key
                ),
            });
        }
    }
    let mut barrels = BTreeMap::new();
    for barrel in &graph.unverified_barrels {
        if barrel.process_qualified {
            return Err("D05603_UNTRUSTED_BARREL_FLAG");
        }
        if barrel.declared_first_layer < 1
            || barrel.declared_last_layer > graph.declared_copper_layers
            || barrel.declared_first_layer > barrel.declared_last_layer
            || barrels.insert(barrel.via_key.as_str(), barrel).is_some()
        {
            return Err("D05603_INVALID_BARREL_INVENTORY");
        }
    }
    let mut qualified_barrels = BTreeSet::new();
    for proof in barrel_proofs {
        if proof.sha256 != current_source_sha256
            || proof.documented_process_reference.trim().is_empty()
            || proof.qualified_transition_reference.trim().is_empty()
            || !qualified_barrels.insert(proof.via_key.as_str())
        {
            return Err("D05603_INVALID_PROCESS_EVIDENCE");
        }
        let barrel = barrels
            .get(proof.via_key.as_str())
            .ok_or("D05603_UNKNOWN_BARREL_PROOF")?;
        if proof.first_layer != barrel.declared_first_layer
            || proof.last_layer != barrel.declared_last_layer
        {
            return Err("D05603_PROCESS_SPAN_MISMATCH");
        }
        let mut layer_pads = BTreeMap::new();
        for &index in &barrel.conductive_pad_nodes {
            let node = graph.nodes.get(index).ok_or("D05603_MISSING_BARREL_NODE")?;
            let layer = node.physical_copper_layer;
            if layer < proof.first_layer
                || layer > proof.last_layer
                || node.source_key != format!("{}:L{layer}", proof.via_key)
                || !matches!(&node.footprint, CopperFootprint::ViaPad { .. })
                || layer_pads.insert(layer, index).is_some()
            {
                return Err("D05603_INVALID_BARREL_LANDING");
            }
        }
        if layer_pads.len() < 2
            || !layer_pads.contains_key(&proof.first_layer)
            || !layer_pads.contains_key(&proof.last_layer)
        {
            return Err("D05603_MISSING_BARREL_ENDPOINT_LANDING");
        }
        // Missing middle pads are legal. A process-reviewed continuous barrel
        // can bridge successive *actual* landing pads without fabricating L3.
        let pads: Vec<_> = layer_pads.values().copied().collect();
        for pair in pads.windows(2) {
            let (first, second) = ordered(pair[0], pair[1]);
            if !seen.insert((first, second)) {
                return Err("D05603_DUPLICATE_TRANSITION_EDGE");
            }
            edges.push(QualifiedDesignEdge {
                first,
                second,
                kind: QualifiedEdgeKind::EvidenceQualifiedBarrel,
                evidence_reference: format!(
                    "{}|{}",
                    proof.documented_process_reference, proof.qualified_transition_reference
                ),
            });
        }
    }
    edges.sort_by_key(|e| (e.first, e.second, e.kind));
    if edges.len() > 150_000 {
        return Err("D05603_QUALIFIED_EDGE_BOUNDS");
    }

    let mut adjacency = vec![Vec::new(); n];
    for (id, e) in edges.iter().enumerate() {
        adjacency[e.first].push((e.second, id));
        adjacency[e.second].push((e.first, id));
    }
    for a in &mut adjacency {
        a.sort_unstable();
    }
    let (components, component_by_node) = components(&adjacency);
    let mut diagnostics = Vec::new();
    if !source_scope_qualified {
        diagnostics.push("PARTIAL_SOURCE_COVERAGE_NO_GLOBAL_OPEN_CLAIM".to_owned());
    }
    let unverified_barrel_keys: Vec<String> = barrels
        .keys()
        .filter(|k| !qualified_barrels.contains(**k))
        .map(|k| (*k).to_owned())
        .collect();
    if !unverified_barrel_keys.is_empty() {
        diagnostics.push("UNVERIFIED_BARREL_CONNECTIONS_EXCLUDED".to_owned());
    }

    let mut net_conflicts = Vec::new();
    let mut conflict_path_nodes = 0usize;
    for (component_id, members) in components.iter().enumerate() {
        let mut nets = BTreeMap::<&str, usize>::new();
        for &index in members {
            if let Some(name) = graph.nodes[index].declared_net.as_deref() {
                nets.entry(name).or_insert(index);
            }
        }
        if nets.len() > 1 {
            let mut it = nets.into_iter();
            let (first_net, first_node) = it.next().ok_or("D05603_EMPTY_NET_SET")?;
            // One BFS per component; never allocate unbounded conflict paths.
            let parents = bfs_parents(&adjacency, first_node);
            for (second_net, second_node) in it {
                if net_conflicts.len() >= 4096 {
                    return Err("D05603_CONFLICT_WITNESS_BOUNDS");
                }
                let path_nodes = path_from_parents(&parents, first_node, second_node)
                    .ok_or("D05603_MISSING_CONFLICT_WITNESS")?;
                if conflict_path_nodes + path_nodes.len() > 100_000 {
                    return Err("D05603_CONFLICT_WITNESS_BOUNDS");
                }
                conflict_path_nodes += path_nodes.len();
                net_conflicts.push(SourceNetConflict {
                    component_id,
                    first_node,
                    second_node,
                    first_net: first_net.to_owned(),
                    second_net: second_net.to_owned(),
                    path_nodes,
                });
            }
        }
    }

    let mut open_witnesses = Vec::new();
    for req in expected_paths {
        if req.net.trim().is_empty() || req.endpoint_source_keys.len() < 2 {
            return Err("D05603_INVALID_REQUIRED_PATH");
        }
        let mut endpoints = Vec::new();
        let mut unique = BTreeSet::new();
        for key in &req.endpoint_source_keys {
            let &index = keys
                .get(key.as_str())
                .ok_or("D05603_MISSING_REQUIRED_ENDPOINT")?;
            if graph.nodes[index].declared_net.as_deref() != Some(req.net.as_str())
                || !unique.insert(index)
            {
                return Err("D05603_INVALID_REQUIRED_ENDPOINT");
            }
            endpoints.push(index);
        }
        let component_ids: Vec<_> = endpoints.iter().map(|&i| component_by_node[i]).collect();
        if component_ids.iter().any(|c| *c != component_ids[0]) {
            open_witnesses.push(OpenWitness {
                net: req.net.clone(),
                endpoints,
                component_ids,
                disposition: if source_scope_qualified && unverified_barrel_keys.is_empty() {
                    OpenDisposition::QualifiedSourceScope
                } else {
                    OpenDisposition::CandidateOnly
                },
            });
        }
    }
    let (bridge_edges, articulation_nodes) = cut_witnesses(&adjacency, &edges);
    Ok(QualificationReport {
        source_sha256: current_source_sha256,
        source_scope_qualified,
        edges,
        components,
        component_by_node,
        net_conflicts,
        open_witnesses,
        bridge_edges,
        articulation_nodes,
        unverified_barrel_keys,
        diagnostics,
    })
}
fn ordered(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}
fn components(adjacency: &[Vec<(usize, usize)>]) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut component_ids = vec![usize::MAX; adjacency.len()];
    let mut components = Vec::new();
    for root in 0..adjacency.len() {
        if component_ids[root] != usize::MAX {
            continue;
        }
        let cid = components.len();
        let mut members = Vec::new();
        let mut queue = VecDeque::from([root]);
        component_ids[root] = cid;
        while let Some(v) = queue.pop_front() {
            members.push(v);
            for &(to, _) in &adjacency[v] {
                if component_ids[to] == usize::MAX {
                    component_ids[to] = cid;
                    queue.push_back(to);
                }
            }
        }
        members.sort_unstable();
        components.push(members);
    }
    (components, component_ids)
}
fn bfs_parents(adjacency: &[Vec<(usize, usize)>], first: usize) -> Vec<usize> {
    let mut parent = vec![usize::MAX; adjacency.len()];
    let mut queue = VecDeque::from([first]);
    parent[first] = first;
    while let Some(v) = queue.pop_front() {
        for &(u, _) in &adjacency[v] {
            if parent[u] == usize::MAX {
                parent[u] = v;
                queue.push_back(u);
            }
        }
    }
    parent
}
fn path_from_parents(parent: &[usize], first: usize, last: usize) -> Option<Vec<usize>> {
    if parent[last] == usize::MAX {
        return None;
    }
    let mut path = vec![last];
    while *path.last()? != first {
        path.push(parent[*path.last()?]);
    }
    path.reverse();
    Some(path)
}
/// Iterative Tarjan: no 50k-deep recursion and no quadratic edge-removal scans.
fn cut_witnesses(
    adjacency: &[Vec<(usize, usize)>],
    edges: &[QualifiedDesignEdge],
) -> (Vec<(usize, usize)>, Vec<usize>) {
    #[derive(Clone, Copy)]
    struct Frame {
        node: usize,
        parent_edge: Option<usize>,
        next: usize,
    }
    let n = adjacency.len();
    let mut time = 0usize;
    let mut discovery = vec![usize::MAX; n];
    let mut low = vec![usize::MAX; n];
    let mut parent = vec![None; n];
    let mut children = vec![0usize; n];
    let mut bridges = Vec::new();
    let mut articulation = BTreeSet::new();
    for root in 0..n {
        if discovery[root] != usize::MAX {
            continue;
        }
        discovery[root] = time;
        low[root] = time;
        time += 1;
        let mut frames = vec![Frame {
            node: root,
            parent_edge: None,
            next: 0,
        }];
        while let Some(top) = frames.last_mut() {
            let v = top.node;
            if top.next < adjacency[v].len() {
                let (to, eid) = adjacency[v][top.next];
                top.next += 1;
                if top.parent_edge == Some(eid) {
                    continue;
                }
                if discovery[to] == usize::MAX {
                    parent[to] = Some(v);
                    children[v] += 1;
                    discovery[to] = time;
                    low[to] = time;
                    time += 1;
                    frames.push(Frame {
                        node: to,
                        parent_edge: Some(eid),
                        next: 0,
                    });
                } else {
                    low[v] = low[v].min(discovery[to]);
                }
            } else {
                let finished = frames.pop().expect("nonempty stack");
                if let Some(p) = parent[v] {
                    low[p] = low[p].min(low[v]);
                    if low[v] > discovery[p] {
                        let e = &edges[finished.parent_edge.expect("tree edge")];
                        bridges.push((e.first, e.second));
                    }
                    if parent[p].is_some() && low[v] >= discovery[p] {
                        articulation.insert(p);
                    }
                } else if children[v] > 1 {
                    articulation.insert(v);
                }
            }
        }
    }
    bridges.sort_unstable();
    (bridges, articulation.into_iter().collect())
}
