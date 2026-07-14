use std::collections::HashSet;

pub type NodeId = usize;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DependencyKind {
    Normal,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyEdge {
    pub target: NodeId,
    pub requested_as: String,
    pub kind: DependencyKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub locator: String,
    pub dependencies: Vec<DependencyEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainLink {
    pub node_id: NodeId,
    pub name: String,
    pub version: String,
    pub locator: String,
    pub requested_as: String,
    pub dependency_kind: DependencyKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyChain {
    pub target_node_id: NodeId,
    pub target_locator: String,
    pub links: Vec<ChainLink>,
    pub warnings: Vec<String>,
}

pub fn package_exists(graph: &DependencyGraph, package_name: &str, package_version: &str) -> bool {
    graph
        .nodes
        .iter()
        .any(|node| node.name == package_name && node.version == package_version)
}

/// Finds every unique path from every matching target instance to a graph root.
///
/// Lockfile adapters resolve dependencies into explicit node-to-node edges before
/// search, so packages with identical names, versions, or descriptors remain
/// distinct when they live at different locators.
pub fn find_dependency_chains(
    graph: &DependencyGraph,
    package_name: &str,
    package_version: &str,
) -> Vec<DependencyChain> {
    #[derive(Clone)]
    struct ParentRef {
        node_id: NodeId,
        requested_as: String,
        dependency_kind: DependencyKind,
    }

    fn emit_chain(
        target_node_id: NodeId,
        graph: &DependencyGraph,
        current_chain: &[ChainLink],
        warnings: Vec<String>,
        seen_chains: &mut HashSet<(NodeId, Vec<NodeId>)>,
        chains: &mut Vec<DependencyChain>,
    ) {
        let key = (
            target_node_id,
            current_chain.iter().map(|link| link.node_id).collect(),
        );
        if !seen_chains.insert(key) {
            return;
        }

        chains.push(DependencyChain {
            target_node_id,
            target_locator: graph.nodes[target_node_id].locator.clone(),
            links: current_chain.to_vec(),
            warnings,
        });
    }

    struct WalkState<'a> {
        graph: &'a DependencyGraph,
        parents: &'a [Vec<ParentRef>],
        target_node_id: NodeId,
        seen_chains: &'a mut HashSet<(NodeId, Vec<NodeId>)>,
        chains: &'a mut Vec<DependencyChain>,
    }

    fn walk(
        current_node_id: NodeId,
        current_chain: &mut Vec<ChainLink>,
        visited: &mut HashSet<NodeId>,
        state: &mut WalkState<'_>,
    ) {
        let parent_refs = &state.parents[current_node_id];
        if parent_refs.is_empty() {
            emit_chain(
                state.target_node_id,
                state.graph,
                current_chain,
                Vec::new(),
                state.seen_chains,
                state.chains,
            );
            return;
        }

        let mut reached_root = false;
        for parent in parent_refs {
            if visited.contains(&parent.node_id) {
                let locator = &state.graph.nodes[parent.node_id].locator;
                emit_chain(
                    state.target_node_id,
                    state.graph,
                    current_chain,
                    vec![format!("Dependency cycle detected at {}", locator)],
                    state.seen_chains,
                    state.chains,
                );
                continue;
            }

            reached_root = true;
            let parent_node = &state.graph.nodes[parent.node_id];
            current_chain.push(ChainLink {
                node_id: parent.node_id,
                name: parent_node.name.clone(),
                version: parent_node.version.clone(),
                locator: parent_node.locator.clone(),
                requested_as: parent.requested_as.clone(),
                dependency_kind: parent.dependency_kind,
            });
            visited.insert(parent.node_id);
            walk(parent.node_id, current_chain, visited, state);
            visited.remove(&parent.node_id);
            current_chain.pop();
        }

        if !reached_root && state.chains.is_empty() {
            emit_chain(
                state.target_node_id,
                state.graph,
                current_chain,
                Vec::new(),
                state.seen_chains,
                state.chains,
            );
        }
    }

    let mut parents = vec![Vec::<ParentRef>::new(); graph.nodes.len()];
    let mut seen_parent_edges = HashSet::new();
    for (parent_node_id, node) in graph.nodes.iter().enumerate() {
        for edge in &node.dependencies {
            if edge.target >= graph.nodes.len() {
                continue;
            }
            let key = (
                edge.target,
                parent_node_id,
                edge.requested_as.clone(),
                edge.kind,
            );
            if seen_parent_edges.insert(key) {
                parents[edge.target].push(ParentRef {
                    node_id: parent_node_id,
                    requested_as: edge.requested_as.clone(),
                    dependency_kind: edge.kind,
                });
            }
        }
    }

    let target_node_ids = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(node_id, node)| {
            (node.name == package_name && node.version == package_version).then_some(node_id)
        })
        .collect::<Vec<_>>();

    let mut chains = Vec::new();
    let mut seen_chains = HashSet::new();
    for target_node_id in target_node_ids {
        let mut visited = HashSet::from([target_node_id]);
        let mut current_chain = Vec::new();
        let mut state = WalkState {
            graph,
            parents: &parents,
            target_node_id,
            seen_chains: &mut seen_chains,
            chains: &mut chains,
        };
        walk(target_node_id, &mut current_chain, &mut visited, &mut state);
    }

    chains
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
