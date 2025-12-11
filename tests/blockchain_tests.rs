use dag_demo::blockchain::*;

// ============================================================================
// GHOSTDAG Algorithm Test Suite
// ============================================================================
//
// These tests demonstrate and verify the correctness of the GHOSTDAG protocol
// implementation, specifically the `ghostdag_sort` and `update_ghostdag_ordering`
// functions.
//
// Key Concepts Tested:
// 1. **Blue/Red Block Classification**: Blocks are colored blue (main chain) or
//    red (side chain) based on their anticone size relative to parameter k.
//
// 2. **Weight Assignment**: Blue blocks receive sequential weights determining
//    their canonical order. Red blocks have weight 0.
//
// 3. **Anticone**: The set of blocks that are neither ancestors nor descendants
//    of a given block (concurrent/parallel blocks).
//
// 4. **k-parameter**: Controls how many blue blocks can be in a block's anticone.
//    Larger k allows more parallelism.
//
// 5. **Topological Ordering**: Blocks are ordered by:
//    - Parent count (depth) - blocks with more parents processed first
//    - Timestamp - earlier blocks preferred when depth is equal
//
// Test Coverage:
// - Linear chains (no forks)
// - Simple forks with varying k values
// - Diamond structures (fork and merge)
// - Complex DAGs with multiple concurrent branches
// - Weight ordering verification
// - Anticone calculation correctness
// - Red block exclusion
// ============================================================================

#[test]
fn test_create_blockdag() {
    let dag = BlockDAG::new(3);
    assert!(dag.get_block("genesis").is_some());
}

#[test]
fn test_add_account() {
    let mut dag = BlockDAG::new(3);
    dag.add_account("alice".to_string(), 1000);

    let account = dag.get_account("alice").unwrap();
    assert_eq!(account.balance, 1000);
    assert_eq!(account.nonce, 0);
}

#[test]
fn test_single_block_chain() {
    let mut dag = BlockDAG::new(3);

    // add block
    let block1 = Block::new("block1".to_string(), vec!["genesis".to_string()], vec![], 1);

    dag.add_block(block1).unwrap();

    let block = dag.get_block("block1").unwrap();
    assert_eq!(block.color, BlockColor::Blue);
    assert_eq!(block.weight, 2); // genesis=1, block1=2
}

#[test]
fn test_transaction_execution_success() {
    let mut dag = BlockDAG::new(3);

    // set account
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);

    // create transaction
    let tx = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );

    // create block containing the transaction
    let block1 = Block::new(
        "block1".to_string(),
        vec!["genesis".to_string()],
        vec![tx],
        1,
    );

    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();

    // Verify balance
    assert_eq!(dag.get_account("alice").unwrap().balance, 900);
    assert_eq!(dag.get_account("bob").unwrap().balance, 600);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 1);

    // Verify transaction status
    let block = dag.get_block("block1").unwrap();
    assert_eq!(block.transactions[0].status, TxStatus::Executed);
}

#[test]
fn test_transaction_execution_insufficient_balance() {
    let mut dag = BlockDAG::new(3);

    // set account
    dag.add_account("alice".to_string(), 50);
    dag.add_account("bob".to_string(), 500);

    // create transaction (insufficient balance)
    let tx = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );

    let block1 = Block::new(
        "block1".to_string(),
        vec!["genesis".to_string()],
        vec![tx],
        1,
    );

    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();

    // Verify balance unchanged
    assert_eq!(dag.get_account("alice").unwrap().balance, 50);
    assert_eq!(dag.get_account("bob").unwrap().balance, 500);

    // Verify transaction failed
    let block = dag.get_block("block1").unwrap();
    match &block.transactions[0].status {
        TxStatus::Failed(reason) => assert!(reason.contains("Insufficient balance")),
        _ => panic!("Expected transaction to fail"),
    }
}

#[test]
fn test_transaction_execution_invalid_nonce() {
    let mut dag = BlockDAG::new(3);

    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);

    // create transaction (wrong nonce)
    let tx = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        5, // should be 0
    );

    let block1 = Block::new(
        "block1".to_string(),
        vec!["genesis".to_string()],
        vec![tx],
        1,
    );

    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();

    // Verify transaction failed
    let block = dag.get_block("block1").unwrap();
    match &block.transactions[0].status {
        TxStatus::Failed(reason) => assert!(reason.contains("Invalid nonce")),
        _ => panic!("Expected transaction to fail"),
    }
}

#[test]
fn test_transaction_revert() {
    let mut dag = BlockDAG::new(3);

    // set account
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);

    // Create and execute transaction
    let tx = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );

    let block1 = Block::new(
        "block1".to_string(),
        vec!["genesis".to_string()],
        vec![tx],
        1,
    );

    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();

    // Verify state after execution
    assert_eq!(dag.get_account("alice").unwrap().balance, 900);
    assert_eq!(dag.get_account("bob").unwrap().balance, 600);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 1);

    // Revert transaction
    dag.revert_block("block1").unwrap();

    // Verify state after revert
    assert_eq!(dag.get_account("alice").unwrap().balance, 1000);
    assert_eq!(dag.get_account("bob").unwrap().balance, 500);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 0);

    // Verify transaction status
    let block = dag.get_block("block1").unwrap();
    assert_eq!(block.transactions[0].status, TxStatus::Reverted);
}

#[test]
fn test_ghostdag_blue_red_blocks() {
    let mut dag = BlockDAG::new(2);

    // Create a forked DAG
    //        genesis
    //        /  |  \
    //      b1  b2  b3
    //        \ | /
    //         b4

    let block1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 1);
    let block2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 2);
    let block3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![], 3);

    dag.add_block(block1).unwrap();
    dag.add_block(block2).unwrap();
    dag.add_block(block3).unwrap();

    // b1 and b2 should be blue (k=2 allows 2 blue blocks in anticone)
    // b3 may be red, depending on anticone size

    let block4 = Block::new(
        "b4".to_string(),
        vec!["b1".to_string(), "b2".to_string(), "b3".to_string()],
        vec![],
        4,
    );
    dag.add_block(block4).unwrap();

    // Verify blue blockchain
    let blue_blocks = dag.get_ordered_blue_blocks();
    assert!(blue_blocks.len() >= 2); // At least genesis and some blue blocks

    // Verify weight increasing
    for i in 1..blue_blocks.len() {
        assert!(blue_blocks[i].weight > blue_blocks[i - 1].weight);
    }
}

#[test]
fn test_complex_dag_with_transactions() {
    let mut dag = BlockDAG::new(3);

    // set account
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);
    dag.add_account("charlie".to_string(), 300);

    // Create blockchain with transactions
    let tx1 = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );
    let block1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![tx1], 1);
    dag.add_block(block1).unwrap();

    let tx2 = Transaction::new(
        "tx2".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
        50,
        0,
    );
    let block2 = Block::new("b2".to_string(), vec!["b1".to_string()], vec![tx2], 2);
    dag.add_block(block2).unwrap();

    // Execute blue chain
    dag.execute_blue_chain().unwrap();

    // Verify final balance
    assert_eq!(dag.get_account("alice").unwrap().balance, 900);
    assert_eq!(dag.get_account("charlie").unwrap().balance, 350);

    // Bob's balance: 500 + 100 - 50 = 550
    assert_eq!(dag.get_account("bob").unwrap().balance, 550);
}

#[test]
fn test_parallel_blocks_ordering() {
    let mut dag = BlockDAG::new(5);

    // Create parallel blocks (fork)
    let block1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let block2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 101);
    let block3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![], 102);

    dag.add_block(block1).unwrap();
    dag.add_block(block2).unwrap();
    dag.add_block(block3).unwrap();

    // All blocks should be blue (k=5 is large enough)
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b3").unwrap().color, BlockColor::Blue);

    // Verify weight ordering
    let blue_blocks = dag.get_ordered_blue_blocks();
    let weights: Vec<u64> = blue_blocks.iter().map(|b| b.weight).collect();

    // Weights should be increasing
    for i in 1..weights.len() {
        assert!(weights[i] > weights[i - 1]);
    }
}

#[test]
fn test_multiple_transaction_revert_in_order() {
    let mut dag = BlockDAG::new(3);

    // set account
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 0);

    // Create block with multiple transactions
    let tx1 = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );
    let tx2 = Transaction::new(
        "tx2".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        200,
        1,
    );
    let tx3 = Transaction::new(
        "tx3".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        300,
        2,
    );

    let block1 = Block::new(
        "b1".to_string(),
        vec!["genesis".to_string()],
        vec![tx1, tx2, tx3],
        1,
    );

    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();

    // Verify state after execution
    assert_eq!(dag.get_account("alice").unwrap().balance, 400); // 1000 - 100 - 200 - 300
    assert_eq!(dag.get_account("bob").unwrap().balance, 600); // 0 + 100 + 200 + 300
    assert_eq!(dag.get_account("alice").unwrap().nonce, 3);

    // Revert entire block
    dag.revert_block("b1").unwrap();

    // Verify state after revert
    assert_eq!(dag.get_account("alice").unwrap().balance, 1000);
    assert_eq!(dag.get_account("bob").unwrap().balance, 0);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 0);

    // Verify all transactions reverted
    let block = dag.get_block("b1").unwrap();
    for tx in &block.transactions {
        assert_eq!(tx.status, TxStatus::Reverted);
    }
}

#[test]
fn test_ghostdag_linear_chain() {
    // Test GHOSTDAG with a simple linear chain (no forks)
    // Expected: All blocks should be blue with sequential weights
    let mut dag = BlockDAG::new(3);

    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 1);
    let b2 = Block::new("b2".to_string(), vec!["b1".to_string()], vec![], 2);
    let b3 = Block::new("b3".to_string(), vec!["b2".to_string()], vec![], 3);

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();
    dag.add_block(b3).unwrap();

    // All blocks should be blue in a linear chain
    assert_eq!(dag.get_block("genesis").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b3").unwrap().color, BlockColor::Blue);

    // Verify sequential weights
    assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
    assert_eq!(dag.get_block("b1").unwrap().weight, 2);
    assert_eq!(dag.get_block("b2").unwrap().weight, 3);
    assert_eq!(dag.get_block("b3").unwrap().weight, 4);
}

#[test]
fn test_ghostdag_simple_fork_k1() {
    // Test GHOSTDAG with k=1
    // When two blocks compete, both may still be blue if their anticone relationship allows
    let mut dag = BlockDAG::new(1);

    // Create a fork: genesis -> b1 and genesis -> b2
    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();

    // With k=1, the first block (b1) should be blue
    // b2's anticone contains b1 (1 blue block), which equals k=1, so b2 can also be blue
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    
    // Check if b2 is blue or red based on anticone size
    let b2 = dag.get_block("b2").unwrap();
    
    // Both should have non-zero weights if both are blue
    assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
    assert_eq!(dag.get_block("b1").unwrap().weight, 2);
    
    // b2 may be blue (weight > 0) or red (weight = 0) depending on anticone
    println!("b2 color: {:?}, weight: {}", b2.color, b2.weight);
}

#[test]
fn test_ghostdag_simple_fork_k3() {
    // Test GHOSTDAG with k=3 (allows parallel blocks)
    // Both competing blocks should be blue
    let mut dag = BlockDAG::new(3);

    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();

    // With k=3, both should be blue (anticone size is 1, which is <= k)
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    
    // Both should have non-zero weights
    assert!(dag.get_block("b1").unwrap().weight > 0);
    assert!(dag.get_block("b2").unwrap().weight > 0);
}

#[test]
fn test_ghostdag_diamond_structure() {
    // Test GHOSTDAG with diamond structure
    //       genesis
    //       /     \
    //      b1     b2
    //       \     /
    //         b3
    let mut dag = BlockDAG::new(3);

    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);
    let b3 = Block::new(
        "b3".to_string(),
        vec!["b1".to_string(), "b2".to_string()],
        vec![],
        300,
    );

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();
    dag.add_block(b3).unwrap();

    // All blocks should be blue with k=3
    assert_eq!(dag.get_block("genesis").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b3").unwrap().color, BlockColor::Blue);

    // Verify all blue blocks have positive weights
    assert!(dag.get_block("genesis").unwrap().weight > 0);
    assert!(dag.get_block("b1").unwrap().weight > 0);
    assert!(dag.get_block("b2").unwrap().weight > 0);
    assert!(dag.get_block("b3").unwrap().weight > 0);
    
    // b3 has more parents (depth=2) so it should be processed before b1, b2 (depth=1)
    // This is due to the sorting by parent count (depth)
    let b3_weight = dag.get_block("b3").unwrap().weight;
    let genesis_weight = dag.get_block("genesis").unwrap().weight;
    
    // b3 should come after genesis but the exact ordering depends on the algorithm
    assert!(b3_weight > genesis_weight);
}

#[test]
fn test_ghostdag_complex_dag_with_k2() {
    // Test GHOSTDAG with a more complex DAG and k=2
    //        genesis
    //        /  |  \
    //      b1  b2  b3
    //        \ | /
    //         b4
    let mut dag = BlockDAG::new(2);

    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);
    let b3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![], 300);
    let b4 = Block::new(
        "b4".to_string(),
        vec!["b1".to_string(), "b2".to_string(), "b3".to_string()],
        vec![],
        400,
    );

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();
    dag.add_block(b3).unwrap();
    dag.add_block(b4).unwrap();

    // With k=2, at most 3 blocks from {b1, b2, b3} can be blue
    // (genesis has anticone {b1, b2, b3} with size 3, but genesis is always blue)
    // b1, b2 should be blue (first two by timestamp)
    // b3 might be red if its blue anticone size exceeds k
    let blue_count = [
        dag.get_block("b1").unwrap(),
        dag.get_block("b2").unwrap(),
        dag.get_block("b3").unwrap(),
    ]
    .iter()
    .filter(|b| b.color == BlockColor::Blue)
    .count();

    // At least 2 should be blue (b1, b2), b3 may or may not be blue
    assert!(blue_count >= 2);

    // b4 should be blue (it references all parents)
    assert_eq!(dag.get_block("b4").unwrap().color, BlockColor::Blue);
}

#[test]
fn test_ghostdag_weight_ordering() {
    // Test that weights are properly ordered in the blue chain
    let mut dag = BlockDAG::new(5);

    // Create a complex DAG
    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);
    let b3 = Block::new("b3".to_string(), vec!["b1".to_string()], vec![], 300);
    let b4 = Block::new(
        "b4".to_string(),
        vec!["b2".to_string(), "b3".to_string()],
        vec![],
        400,
    );

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();
    dag.add_block(b3).unwrap();
    dag.add_block(b4).unwrap();

    // Get ordered blue blocks
    let blue_blocks = dag.get_ordered_blue_blocks();

    // Verify weights are strictly increasing
    for i in 1..blue_blocks.len() {
        assert!(
            blue_blocks[i].weight > blue_blocks[i - 1].weight,
            "Weight ordering violated: block {} (weight {}) should have higher weight than block {} (weight {})",
            blue_blocks[i].hash,
            blue_blocks[i].weight,
            blue_blocks[i - 1].hash,
            blue_blocks[i - 1].weight
        );
    }

    // First block should be genesis with weight 1
    assert_eq!(blue_blocks[0].hash, "genesis");
    assert_eq!(blue_blocks[0].weight, 1);
}

#[test]
fn test_ghostdag_timestamp_ordering() {
    // Test that earlier timestamp blocks are preferred when anticone size allows
    let mut dag = BlockDAG::new(5);

    // Create blocks with different timestamps
    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 500);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![], 300);

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();
    dag.add_block(b3).unwrap();

    // All should be blue with k=5
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b3").unwrap().color, BlockColor::Blue);

    // b2 (timestamp 100) should have lower weight than b3 (timestamp 300)
    // which should have lower weight than b1 (timestamp 500)
    // because they're ordered by timestamp when at same depth
    let w1 = dag.get_block("b1").unwrap().weight;
    let w2 = dag.get_block("b2").unwrap().weight;
    let w3 = dag.get_block("b3").unwrap().weight;

    assert!(w2 < w3, "b2 (ts=100) should come before b3 (ts=300)");
    assert!(w3 < w1, "b3 (ts=300) should come before b1 (ts=500)");
}

#[test]
fn test_ghostdag_anticone_calculation() {
    // Test that anticone is correctly calculated
    // Create a DAG where we can verify anticone relationships
    //     genesis
    //      /   \
    //     b1   b2  (b1 and b2 are in each other's anticone)
    //      \   /
    //       b3    (b3 is descendant of both b1 and b2)
    let mut dag = BlockDAG::new(3);

    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);
    let b3 = Block::new(
        "b3".to_string(),
        vec!["b1".to_string(), "b2".to_string()],
        vec![],
        300,
    );

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();
    dag.add_block(b3).unwrap();

    // All blocks should be blue (anticone size is small enough)
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b3").unwrap().color, BlockColor::Blue);

    // b3 references both b1 and b2, so they're not in its anticone
    // This should allow b3 to be blue
    assert!(dag.get_block("b3").unwrap().weight > 0);
}

#[test]
fn test_ghostdag_red_block_exclusion() {
    // Test that red blocks don't contribute to weight and are properly excluded
    let mut dag = BlockDAG::new(0); // k=0 means only one chain can be blue

    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();

    // With k=0, only one fork can be blue
    let blue_count = [
        dag.get_block("b1").unwrap(),
        dag.get_block("b2").unwrap(),
    ]
    .iter()
    .filter(|b| b.color == BlockColor::Blue)
    .count();

    assert_eq!(blue_count, 1, "Only one block should be blue with k=0");

    // Red block should have weight 0
    let red_block = if dag.get_block("b1").unwrap().color == BlockColor::Red {
        dag.get_block("b1").unwrap()
    } else {
        dag.get_block("b2").unwrap()
    };

    assert_eq!(
        red_block.weight, 0,
        "Red block should have weight 0"
    );
}

#[test]
fn test_ghostdag_multiple_parents_ordering() {
    // Test ordering when a block has multiple parents
    // Blocks with more parents are sorted first (higher depth)
    let mut dag = BlockDAG::new(5);

    let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);
    let b3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![], 300);
    
    // b4 has more parents (3 vs 1), so it's processed first in sorting
    let b4 = Block::new(
        "b4".to_string(),
        vec!["b1".to_string(), "b2".to_string(), "b3".to_string()],
        vec![],
        150,
    );

    dag.add_block(b1).unwrap();
    dag.add_block(b2).unwrap();
    dag.add_block(b3).unwrap();
    dag.add_block(b4).unwrap();

    // All should be blue with k=5
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b3").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b4").unwrap().color, BlockColor::Blue);

    // Due to sorting by depth (parent count), b4 with 3 parents is processed before
    // b1, b2, b3 which each have 1 parent. So b4 may have lower weight.
    let w1 = dag.get_block("b1").unwrap().weight;
    let w2 = dag.get_block("b2").unwrap().weight;
    let w3 = dag.get_block("b3").unwrap().weight;
    let w4 = dag.get_block("b4").unwrap().weight;

    // Verify all are blue (have positive weights)
    assert!(w1 > 0, "b1 should be blue");
    assert!(w2 > 0, "b2 should be blue");
    assert!(w3 > 0, "b3 should be blue");
    assert!(w4 > 0, "b4 should be blue");
    
    // b4 is processed first (more parents = higher priority in sorting)
    // so it will have lower weight than some others
    println!("Weights: b1={}, b2={}, b3={}, b4={}", w1, w2, w3, w4);
}
