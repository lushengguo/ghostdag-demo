use std::collections::{HashMap, HashSet, VecDeque};

/// Represents a node in the DAG
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    pub id: String,
    pub data: String,
}

impl Node {
    pub fn new(id: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            data: data.into(),
        }
    }
}

/// Directed Acyclic Graph (DAG) structure
#[derive(Debug)]
pub struct Dag {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, HashSet<String>>, // node_id -> set of child node_ids
}

impl Dag {
    /// Create a new empty DAG
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a node to the DAG
    pub fn add_node(&mut self, node: Node) -> Result<(), String> {
        if self.nodes.contains_key(&node.id) {
            return Err(format!("Node with id '{}' already exists", node.id));
        }
        let node_id = node.id.clone();
        self.nodes.insert(node_id.clone(), node);
        self.edges.insert(node_id, HashSet::new());
        Ok(())
    }

    /// Add an edge from one node to another
    /// Returns an error if adding the edge would create a cycle
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<(), String> {
        if !self.nodes.contains_key(from) {
            return Err(format!("Source node '{}' does not exist", from));
        }
        if !self.nodes.contains_key(to) {
            return Err(format!("Target node '{}' does not exist", to));
        }

        // Check if adding this edge would create a cycle
        if self.would_create_cycle(from, to) {
            return Err(format!(
                "Adding edge from '{}' to '{}' would create a cycle",
                from, to
            ));
        }

        self.edges.get_mut(from).unwrap().insert(to.to_string());
        Ok(())
    }

    /// Check if adding an edge would create a cycle using DFS
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // If there's already a path from 'to' to 'from', adding 'from' -> 'to' creates a cycle
        self.has_path(to, from)
    }

    /// Check if there's a path from start to end using BFS
    fn has_path(&self, start: &str, end: &str) -> bool {
        if start == end {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.edges.get(current) {
                for child in children {
                    if child == end {
                        return true;
                    }
                    if !visited.contains(child.as_str()) {
                        visited.insert(child.as_str());
                        queue.push_back(child);
                    }
                }
            }
        }

        false
    }

    /// Get all nodes in the DAG
    pub fn get_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    /// Get children of a node
    pub fn get_children(&self, node_id: &str) -> Option<Vec<&Node>> {
        self.edges.get(node_id).map(|children| {
            children
                .iter()
                .filter_map(|id| self.nodes.get(id))
                .collect()
        })
    }

    /// Perform topological sort on the DAG
    /// Returns nodes in topological order (dependencies before dependents)
    pub fn topological_sort(&self) -> Result<Vec<Node>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        
        // Initialize in-degree for all nodes
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
        }

        // Calculate in-degree for each node
        for children in self.edges.values() {
            for child in children {
                *in_degree.get_mut(child).unwrap() += 1;
            }
        }

        // Queue nodes with in-degree 0
        let mut queue = VecDeque::new();
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        let mut result = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            result.push(self.nodes.get(&node_id).unwrap().clone());

            // Reduce in-degree of children
            if let Some(children) = self.edges.get(&node_id) {
                for child in children {
                    let degree = in_degree.get_mut(child).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(child.clone());
                    }
                }
            }
        }

        // If not all nodes are in result, there's a cycle (shouldn't happen with our validation)
        if result.len() != self.nodes.len() {
            return Err("DAG contains a cycle".to_string());
        }

        Ok(result)
    }

    /// Get the number of nodes in the DAG
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in the DAG
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|set| set.len()).sum()
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_dag() {
        let dag = Dag::new();
        assert_eq!(dag.node_count(), 0);
        assert_eq!(dag.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut dag = Dag::new();
        let node = Node::new("A", "Node A");
        assert!(dag.add_node(node).is_ok());
        assert_eq!(dag.node_count(), 1);
    }

    #[test]
    fn test_add_duplicate_node() {
        let mut dag = Dag::new();
        let node1 = Node::new("A", "Node A");
        let node2 = Node::new("A", "Another Node A");
        
        assert!(dag.add_node(node1).is_ok());
        assert!(dag.add_node(node2).is_err());
        assert_eq!(dag.node_count(), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        dag.add_node(Node::new("B", "Node B")).unwrap();
        
        assert!(dag.add_edge("A", "B").is_ok());
        assert_eq!(dag.edge_count(), 1);
    }

    #[test]
    fn test_add_edge_nonexistent_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        
        assert!(dag.add_edge("A", "B").is_err());
        assert!(dag.add_edge("C", "A").is_err());
    }

    #[test]
    fn test_cycle_detection_simple() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        dag.add_node(Node::new("B", "Node B")).unwrap();
        
        dag.add_edge("A", "B").unwrap();
        // This would create a cycle: A -> B -> A
        assert!(dag.add_edge("B", "A").is_err());
    }

    #[test]
    fn test_cycle_detection_complex() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        dag.add_node(Node::new("B", "Node B")).unwrap();
        dag.add_node(Node::new("C", "Node C")).unwrap();
        
        dag.add_edge("A", "B").unwrap();
        dag.add_edge("B", "C").unwrap();
        // This would create a cycle: A -> B -> C -> A
        assert!(dag.add_edge("C", "A").is_err());
    }

    #[test]
    fn test_topological_sort_linear() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        dag.add_node(Node::new("B", "Node B")).unwrap();
        dag.add_node(Node::new("C", "Node C")).unwrap();
        
        dag.add_edge("A", "B").unwrap();
        dag.add_edge("B", "C").unwrap();
        
        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        
        // A should come before B, B should come before C
        let positions: HashMap<String, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id.clone(), i))
            .collect();
        
        assert!(positions["A"] < positions["B"]);
        assert!(positions["B"] < positions["C"]);
    }

    #[test]
    fn test_topological_sort_diamond() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        dag.add_node(Node::new("B", "Node B")).unwrap();
        dag.add_node(Node::new("C", "Node C")).unwrap();
        dag.add_node(Node::new("D", "Node D")).unwrap();
        
        // Diamond shape: A -> B -> D, A -> C -> D
        dag.add_edge("A", "B").unwrap();
        dag.add_edge("A", "C").unwrap();
        dag.add_edge("B", "D").unwrap();
        dag.add_edge("C", "D").unwrap();
        
        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted.len(), 4);
        
        let positions: HashMap<String, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id.clone(), i))
            .collect();
        
        // A must come before all others
        assert!(positions["A"] < positions["B"]);
        assert!(positions["A"] < positions["C"]);
        assert!(positions["A"] < positions["D"]);
        
        // B and C must come before D
        assert!(positions["B"] < positions["D"]);
        assert!(positions["C"] < positions["D"]);
    }

    #[test]
    fn test_topological_sort_empty() {
        let dag = Dag::new();
        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted.len(), 0);
    }

    #[test]
    fn test_topological_sort_single_node() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        
        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].id, "A");
    }

    #[test]
    fn test_get_children() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        dag.add_node(Node::new("B", "Node B")).unwrap();
        dag.add_node(Node::new("C", "Node C")).unwrap();
        
        dag.add_edge("A", "B").unwrap();
        dag.add_edge("A", "C").unwrap();
        
        let children = dag.get_children("A").unwrap();
        assert_eq!(children.len(), 2);
        
        let child_ids: HashSet<&str> = children.iter().map(|n| n.id.as_str()).collect();
        assert!(child_ids.contains("B"));
        assert!(child_ids.contains("C"));
    }

    #[test]
    fn test_get_children_no_children() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        
        let children = dag.get_children("A").unwrap();
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_get_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("A", "Node A")).unwrap();
        dag.add_node(Node::new("B", "Node B")).unwrap();
        
        let nodes = dag.get_nodes();
        assert_eq!(nodes.len(), 2);
    }
}
