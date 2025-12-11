use dag_demo::{Dag, Node};

fn main() {
    println!("=== DAG Protocol Demo ===\n");

    // Create a new DAG
    let mut dag = Dag::new();

    // Example: Task dependency graph
    println!("1. Creating task dependency graph:");
    dag.add_node(Node::new("install", "Install dependencies"))
        .unwrap();
    dag.add_node(Node::new("compile", "Compile source code"))
        .unwrap();
    dag.add_node(Node::new("test", "Run tests")).unwrap();
    dag.add_node(Node::new("package", "Package application"))
        .unwrap();
    dag.add_node(Node::new("deploy", "Deploy to production"))
        .unwrap();

    // Define dependencies
    dag.add_edge("install", "compile").unwrap(); // Compile depends on install
    dag.add_edge("compile", "test").unwrap(); // Test depends on compile
    dag.add_edge("compile", "package").unwrap(); // Package depends on compile
    dag.add_edge("test", "deploy").unwrap(); // Deploy depends on test
    dag.add_edge("package", "deploy").unwrap(); // Deploy depends on package

    println!("   Nodes: {}", dag.node_count());
    println!("   Edges: {}\n", dag.edge_count());

    // Show topological sort (execution order)
    println!("2. Valid execution order (topological sort):");
    match dag.topological_sort() {
        Ok(sorted) => {
            for (i, node) in sorted.iter().enumerate() {
                println!("   Step {}: {} - {}", i + 1, node.id, node.data);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n3. Demonstrating cycle prevention:");
    // Try to create a cycle
    match dag.add_edge("deploy", "install") {
        Ok(_) => println!("   Edge added (unexpected!)"),
        Err(e) => println!("   ✓ Cycle prevented: {}", e),
    }

    // Show children of a node
    println!("\n4. Dependencies of 'compile' task:");
    if let Some(children) = dag.get_children("compile") {
        for child in children {
            println!("   → {} ({})", child.id, child.data);
        }
    }

    println!("\n=== Demo Complete ===");
}
