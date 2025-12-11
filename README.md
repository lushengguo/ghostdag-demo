# ghostdag-demo

A minimal Directed Acyclic Graph (DAG) protocol implementation in Rust, including a DAG-based blockchain demonstration using the GHOSTDAG protocol.

## Overview

This library demonstrates the core concepts of DAG protocols with clean, focused implementations:

### 1. Basic DAG Structure
A directed graph with no cycles, commonly used in:
- Task scheduling and dependency resolution
- Version control systems (Git)
- Data processing pipelines
- Blockchain consensus mechanisms

### 2. DAG-based Blockchain (GHOSTDAG Protocol)
A blockchain implementation based on DAG structure that allows parallel block creation:
- **Parallel blocks**: Multiple blocks can reference the same parent(s)
- **Blue/Red coloring**: GHOSTDAG algorithm determines main chain blocks (blue) vs. side blocks (red)
- **Transaction execution**: Transactions are executed based on blue chain ordering
- **Transaction rollback**: Support for reverting transactions when needed
- **Weight-based ordering**: Blocks are ordered by their cumulative weight in the DAG

## Features

### Basic DAG
- **Node Management**: Add nodes with unique identifiers and associated data
- **Edge Management**: Create directed edges between nodes
- **Cycle Detection**: Automatically prevents cycle creation to maintain acyclic property
- **Topological Sort**: Order nodes respecting dependency relationships
- **Path Finding**: Check connectivity between nodes

### Blockchain DAG
- **Block Structure**: Blocks with multiple parent references (DAG structure)
- **GHOSTDAG Algorithm**: Implements k-cluster blue/red block classification
- **Transaction Management**: 
  - Transaction validation (balance, nonce checking)
  - Execution with proper state updates
  - Failure handling (insufficient balance, invalid nonce)
  - Transaction rollback support
- **Account System**: Simple balance and nonce tracking
- **Weight-based Ordering**: Deterministic ordering of blocks in the DAG

## Usage

### Basic DAG Usage

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

### Blockchain DAG Usage

```rust
use dag_demo::blockchain::*;

fn main() {
    // Create BlockDAG with k=3 (allows up to 3 blue blocks in anticone)
    let mut dag = BlockDAG::new(3);
    
    // Setup accounts
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);
    
    // Create parallel blocks (simulating concurrent mining)
    let tx1 = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );
    
    let block1 = Block::new(
        "b1".to_string(),
        vec!["genesis".to_string()],
        vec![tx1],
        100,
    );
    dag.add_block(block1).unwrap();
    
    // GHOSTDAG automatically classifies blocks as Blue or Red
    let ordered_blocks = dag.get_ordered_blue_blocks();
    for block in ordered_blocks {
        println!("{}: {:?} (weight: {})", 
            block.hash, block.color, block.weight);
    }
    
    // Execute transactions in blue chain order
    dag.execute_blue_chain().unwrap();
    
    // Check account balances
    let alice = dag.get_account("alice").unwrap();
    println!("Alice: {} (nonce: {})", alice.balance, alice.nonce);
    
    // Revert transactions if needed
    dag.revert_block("b1").unwrap();
}
```

## GHOSTDAG Protocol

The GHOSTDAG (Greedy Heaviest-Observed Sub-DAG) protocol is implemented with the following features:

1. **k-cluster parameter**: Controls the maximum number of blue blocks allowed in the anticone
2. **Blue block set**: Main chain blocks that contribute to transaction ordering
3. **Red block set**: Side blocks that don't contribute to the main chain
4. **Weight calculation**: Each blue block contributes weight to determine canonical ordering
5. **Anticone calculation**: Blocks that are neither ancestors nor descendants

### Key Concepts

- **Blue blocks** 🔵: Blocks in the main chain, transactions are executed
- **Red blocks** 🔴: Side blocks, transactions are not executed (or executed later)
- **Weight**: Cumulative count of blue blocks up to and including current block
- **Anticone**: Set of blocks that are concurrent (neither ancestors nor descendants)

## Testing

Run all tests:

```bash
cargo test
```

Run blockchain-specific tests:

```bash
cargo test --test blockchain_tests
```

### Test Coverage

**Basic DAG tests:**
- Basic node and edge operations
- Cycle detection (simple and complex cases)
- Topological sorting (linear, diamond, and edge cases)
- Children and node retrieval

**Blockchain tests:**
- Block creation and DAG structure
- GHOSTDAG blue/red block classification
- Transaction execution (success cases)
- Transaction failure (insufficient balance, invalid nonce)
- Transaction revert and rollback
- Multi-transaction blocks
- Parallel block ordering

## Examples

Run the basic DAG example:

```bash
cargo run --example basic_usage
```

Run the blockchain demo:

```bash
cargo run --example blockchain_demo
```

The blockchain demo showcases:
- Parallel block creation (DAG structure)
- GHOSTDAG blue/red classification
- Transaction execution in weight order
- Failed transaction handling
- Transaction rollback mechanism

## Core Properties

### DAG Properties
1. **Acyclic**: No node can reach itself by following directed edges
2. **Directed**: Edges have a specific direction (parent -> child)
3. **Topological Ordering**: Nodes can be linearly ordered such that for every edge (u,v), u comes before v

### Blockchain Properties
1. **DAG Structure**: Blocks can have multiple parents, allowing parallel mining
2. **Deterministic Ordering**: GHOSTDAG ensures canonical ordering despite parallelism
3. **Transaction Safety**: Proper nonce and balance validation
4. **Reversibility**: Support for transaction rollback

## Build

```bash
cargo build
```
