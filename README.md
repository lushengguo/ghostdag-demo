# dag-demo

A minimal Directed Acyclic Graph (DAG) protocol implementation in Rust.

## Overview

This library demonstrates the core concepts of a DAG protocol with a clean, focused implementation. A DAG is a directed graph with no cycles, commonly used in:
- Task scheduling and dependency resolution
- Version control systems (Git)
- Data processing pipelines
- Blockchain consensus mechanisms

## Features

- **Node Management**: Add nodes with unique identifiers and associated data
- **Edge Management**: Create directed edges between nodes
- **Cycle Detection**: Automatically prevents cycle creation to maintain acyclic property
- **Topological Sort**: Order nodes respecting dependency relationships
- **Path Finding**: Check connectivity between nodes

## Usage

```rust
use dag_demo::{Dag, Node};

fn main() {
    let mut dag = Dag::new();
    
    // Add nodes
    dag.add_node(Node::new("A", "Task A")).unwrap();
    dag.add_node(Node::new("B", "Task B")).unwrap();
    dag.add_node(Node::new("C", "Task C")).unwrap();
    
    // Add edges (A -> B, A -> C, B -> C)
    dag.add_edge("A", "B").unwrap();
    dag.add_edge("A", "C").unwrap();
    dag.add_edge("B", "C").unwrap();
    
    // Get topological sort (valid execution order)
    let sorted = dag.topological_sort().unwrap();
    for node in sorted {
        println!("{}: {}", node.id, node.data);
    }
}
```

## Core DAG Properties

1. **Acyclic**: No node can reach itself by following directed edges
2. **Directed**: Edges have a specific direction (parent -> child)
3. **Topological Ordering**: Nodes can be linearly ordered such that for every edge (u,v), u comes before v

## Testing

Run the test suite:

```bash
cargo test
```

The test suite covers:
- Basic node and edge operations
- Cycle detection (simple and complex cases)
- Topological sorting (linear, diamond, and edge cases)
- Children and node retrieval

## Build

```bash
cargo build
```
