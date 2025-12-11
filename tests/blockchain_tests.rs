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
    
    // 添加区块
    let block1 = Block::new(
        "block1".to_string(),
        vec!["genesis".to_string()],
        vec![],
        1,
    );
    
    dag.add_block(block1).unwrap();
    
    let block = dag.get_block("block1").unwrap();
    assert_eq!(block.color, BlockColor::Blue);
    assert_eq!(block.weight, 2); // genesis=1, block1=2
}

#[test]
fn test_transaction_execution_success() {
    let mut dag = BlockDAG::new(3);
    
    // 设置账户
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);
    
    // 创建交易
    let tx = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        0,
    );
    
    // 创建包含交易的区块
    let block1 = Block::new(
        "block1".to_string(),
        vec!["genesis".to_string()],
        vec![tx],
        1,
    );
    
    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();
    
    // 验证余额
    assert_eq!(dag.get_account("alice").unwrap().balance, 900);
    assert_eq!(dag.get_account("bob").unwrap().balance, 600);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 1);
    
    // 验证交易状态
    let block = dag.get_block("block1").unwrap();
    assert_eq!(block.transactions[0].status, TxStatus::Executed);
}

#[test]
fn test_transaction_execution_insufficient_balance() {
    let mut dag = BlockDAG::new(3);
    
    // 设置账户
    dag.add_account("alice".to_string(), 50);
    dag.add_account("bob".to_string(), 500);
    
    // 创建交易（余额不足）
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
    
    // 验证余额未改变
    assert_eq!(dag.get_account("alice").unwrap().balance, 50);
    assert_eq!(dag.get_account("bob").unwrap().balance, 500);
    
    // 验证交易失败
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
    
    // 创建交易（错误的 nonce）
    let tx = Transaction::new(
        "tx1".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        100,
        5, // 应该是 0
    );
    
    let block1 = Block::new(
        "block1".to_string(),
        vec!["genesis".to_string()],
        vec![tx],
        1,
    );
    
    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();
    
    // 验证交易失败
    let block = dag.get_block("block1").unwrap();
    match &block.transactions[0].status {
        TxStatus::Failed(reason) => assert!(reason.contains("Invalid nonce")),
        _ => panic!("Expected transaction to fail"),
    }
}

#[test]
fn test_transaction_revert() {
    let mut dag = BlockDAG::new(3);
    
    // 设置账户
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);
    
    // 创建并执行交易
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
    
    // 验证执行后的状态
    assert_eq!(dag.get_account("alice").unwrap().balance, 900);
    assert_eq!(dag.get_account("bob").unwrap().balance, 600);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 1);
    
    // 回滚交易
    dag.revert_block("block1").unwrap();
    
    // 验证回滚后的状态
    assert_eq!(dag.get_account("alice").unwrap().balance, 1000);
    assert_eq!(dag.get_account("bob").unwrap().balance, 500);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 0);
    
    // 验证交易状态
    let block = dag.get_block("block1").unwrap();
    assert_eq!(block.transactions[0].status, TxStatus::Reverted);
}

#[test]
fn test_ghostdag_blue_red_blocks() {
    let mut dag = BlockDAG::new(2);
    
    // 创建一个分叉的 DAG
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
    
    // b1 和 b2 应该是蓝色（k=2 允许 2 个反锥体中的蓝色区块）
    // b3 可能是红色，取决于反锥体大小
    
    let block4 = Block::new(
        "b4".to_string(),
        vec!["b1".to_string(), "b2".to_string(), "b3".to_string()],
        vec![],
        4,
    );
    dag.add_block(block4).unwrap();
    
    // 验证蓝色区块链
    let blue_blocks = dag.get_ordered_blue_blocks();
    assert!(blue_blocks.len() >= 2); // 至少有 genesis 和一些蓝色区块
    
    // 验证权重递增
    for i in 1..blue_blocks.len() {
        assert!(blue_blocks[i].weight > blue_blocks[i - 1].weight);
    }
}

#[test]
fn test_complex_dag_with_transactions() {
    let mut dag = BlockDAG::new(3);
    
    // 设置账户
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 500);
    dag.add_account("charlie".to_string(), 300);
    
    // 创建包含交易的区块链
    let tx1 = Transaction::new("tx1".to_string(), "alice".to_string(), "bob".to_string(), 100, 0);
    let block1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![tx1], 1);
    dag.add_block(block1).unwrap();
    
    let tx2 = Transaction::new("tx2".to_string(), "bob".to_string(), "charlie".to_string(), 50, 0);
    let block2 = Block::new("b2".to_string(), vec!["b1".to_string()], vec![tx2], 2);
    dag.add_block(block2).unwrap();
    
    // 执行蓝色链
    dag.execute_blue_chain().unwrap();
    
    // 验证最终余额
    assert_eq!(dag.get_account("alice").unwrap().balance, 900);
    assert_eq!(dag.get_account("charlie").unwrap().balance, 350);
    
    // Bob 的余额：500 + 100 - 50 = 550
    assert_eq!(dag.get_account("bob").unwrap().balance, 550);
}

#[test]
fn test_parallel_blocks_ordering() {
    let mut dag = BlockDAG::new(5);
    
    // 创建并行区块（分叉）
    let block1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
    let block2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 101);
    let block3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![], 102);
    
    dag.add_block(block1).unwrap();
    dag.add_block(block2).unwrap();
    dag.add_block(block3).unwrap();
    
    // 所有区块都应该是蓝色（k=5 足够大）
    assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);
    assert_eq!(dag.get_block("b3").unwrap().color, BlockColor::Blue);
    
    // 验证权重排序
    let blue_blocks = dag.get_ordered_blue_blocks();
    let weights: Vec<u64> = blue_blocks.iter().map(|b| b.weight).collect();
    
    // 权重应该是递增的
    for i in 1..weights.len() {
        assert!(weights[i] > weights[i - 1]);
    }
}

#[test]
fn test_multiple_transaction_revert_in_order() {
    let mut dag = BlockDAG::new(3);
    
    // 设置账户
    dag.add_account("alice".to_string(), 1000);
    dag.add_account("bob".to_string(), 0);
    
    // 创建多笔交易的区块
    let tx1 = Transaction::new("tx1".to_string(), "alice".to_string(), "bob".to_string(), 100, 0);
    let tx2 = Transaction::new("tx2".to_string(), "alice".to_string(), "bob".to_string(), 200, 1);
    let tx3 = Transaction::new("tx3".to_string(), "alice".to_string(), "bob".to_string(), 300, 2);
    
    let block1 = Block::new(
        "b1".to_string(),
        vec!["genesis".to_string()],
        vec![tx1, tx2, tx3],
        1,
    );
    
    dag.add_block(block1).unwrap();
    dag.execute_blue_chain().unwrap();
    
    // 验证执行后状态
    assert_eq!(dag.get_account("alice").unwrap().balance, 400);  // 1000 - 100 - 200 - 300
    assert_eq!(dag.get_account("bob").unwrap().balance, 600);    // 0 + 100 + 200 + 300
    assert_eq!(dag.get_account("alice").unwrap().nonce, 3);
    
    // 回滚整个区块
    dag.revert_block("b1").unwrap();
    
    // 验证回滚后状态
    assert_eq!(dag.get_account("alice").unwrap().balance, 1000);
    assert_eq!(dag.get_account("bob").unwrap().balance, 0);
    assert_eq!(dag.get_account("alice").unwrap().nonce, 0);
    
    // 验证所有交易都被回滚
    let block = dag.get_block("b1").unwrap();
    for tx in &block.transactions {
        assert_eq!(tx.status, TxStatus::Reverted);
    }
}
