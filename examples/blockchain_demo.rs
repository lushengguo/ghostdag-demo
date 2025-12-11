use dag_demo::blockchain::*;

fn main() {
    println!("=== DAG-based Blockchain Demo (GHOSTDAG Protocol) ===\n");

    // Create BlockDAG, k=3 allows up to 3 blue blocks in anticone
    let mut dag = BlockDAG::new(3);

    println!("1. Setting up accounts:");
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);
    dag.add_account("charlie".to_string(), 300);
    
    println!("   Alice: 1000, Bob: 500, Charlie: 300\n");

    println!("2. Creating a DAG structure with parallel blocks:");
    println!("   Structure:");
    println!("          genesis");
    println!("          /  |  \\");
    println!("        b1  b2  b3");
    println!("          \\  |  /");
    println!("            b4\n");

    // Create parallel blocks (simulating concurrent mining by different miners)
    let tx1 = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );
    let block1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![tx1], 100);
    dag.add_block(block1).unwrap();

    let tx2 = Transaction::new(
        "tx2".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
        50,
        0,
    );
    let block2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![tx2], 101);
    dag.add_block(block2).unwrap();

    let tx3 = Transaction::new(
        "tx3".to_string(),
        "charlie".to_string(),
        "alice".to_string(),
        30,
        0,
    );
    let block3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![tx3], 102);
    dag.add_block(block3).unwrap();

    // Create merge block
    let tx4 = Transaction::new(
        "tx4".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        50,
        1,
    );
    let block4 = Block::new(
        "b4".to_string(),
        vec!["b1".to_string(), "b2".to_string(), "b3".to_string()],
        vec![tx4],
        103,
    );
    dag.add_block(block4).unwrap();

    println!("3. GHOSTDAG Blue/Red Block Classification:");
    let all_blocks = dag.get_all_blocks();
    for block in all_blocks {
        if block.hash == "genesis" {
            continue;
        }
        let color = match block.color {
            BlockColor::Blue => "🔵 Blue",
            BlockColor::Red => "🔴 Red",
        };
        println!("   Block {}: {} (weight: {})", block.hash, color, block.weight);
    }

    println!("\n4. Ordered Blue Chain (by weight):");
    let blue_blocks = dag.get_ordered_blue_blocks();
    for block in &blue_blocks {
        println!("   {} -> weight: {}", block.hash, block.weight);
    }

    println!("\n5. Executing transactions in blue chain order:");
    dag.execute_blue_chain().unwrap();

    // 重新获取蓝色区块以显示交易执行状态
    let blue_blocks = dag.get_ordered_blue_blocks();
    for block in blue_blocks {
        if !block.transactions.is_empty() {
            println!("   Block {}:", block.hash);
            for tx in &block.transactions {
                let status = match &tx.status {
                    TxStatus::Executed => "✓ Executed",
                    TxStatus::Failed(reason) => &format!("✗ Failed: {}", reason),
                    TxStatus::Pending => "⧖ Pending",
                    TxStatus::Reverted => "↶ Reverted",
                };
                println!("     {} ({} -> {}, amount: {}): {}", 
                    tx.id, tx.from, tx.to, tx.amount, status);
            }
        }
    }

    println!("\n6. Final account balances:");
    if let Some(alice) = dag.get_account("alice") {
        println!("   Alice: {} (nonce: {})", alice.balance, alice.nonce);
    }
    if let Some(bob) = dag.get_account("bob") {
        println!("   Bob: {} (nonce: {})", bob.balance, bob.nonce);
    }
    if let Some(charlie) = dag.get_account("charlie") {
        println!("   Charlie: {} (nonce: {})", charlie.balance, charlie.nonce);
    }

    println!("\n7. Demonstrating transaction revert:");
    println!("   Reverting block b1...");
    
    let alice_before = dag.get_account("alice").unwrap().balance;
    let bob_before = dag.get_account("bob").unwrap().balance;
    
    dag.revert_block("b1").unwrap();
    
    let alice_after = dag.get_account("alice").unwrap().balance;
    let bob_after = dag.get_account("bob").unwrap().balance;
    
    println!("   Alice: {} -> {}", alice_before, alice_after);
    println!("   Bob: {} -> {}", bob_before, bob_after);
    
    let block = dag.get_block("b1").unwrap();
    for tx in &block.transactions {
        println!("   Transaction {} status: {:?}", tx.id, tx.status);
    }

    println!("\n8. Demonstrating failed transaction:");
    let mut dag2 = BlockDAG::new(3);
    dag2.add_account("poor_alice".to_string(), 10);
    dag2.add_account("rich_bob".to_string(), 1000);
    
    // 尝试转账超过余额的金额
    let tx_fail = Transaction::new(
        "tx_fail".to_string(),
        "poor_alice".to_string(),
        "rich_bob".to_string(),
        100,
        0,
    );
    
    let block_fail = Block::new(
        "b_fail".to_string(),
        vec!["genesis".to_string()],
        vec![tx_fail],
        200,
    );
    
    dag2.add_block(block_fail).unwrap();
    dag2.execute_blue_chain().unwrap();
    
    let block = dag2.get_block("b_fail").unwrap();
    match &block.transactions[0].status {
        TxStatus::Failed(reason) => println!("   ✗ Transaction failed as expected: {}", reason),
        _ => println!("   Unexpected transaction status"),
    }
    
    println!("   Poor Alice still has: {}", dag2.get_account("poor_alice").unwrap().balance);
    println!("   Rich Bob still has: {}", dag2.get_account("rich_bob").unwrap().balance);

    println!("\n=== Demo Complete ===");
}
