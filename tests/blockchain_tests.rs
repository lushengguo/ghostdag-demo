use dag_demo::blockchain::*;

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
