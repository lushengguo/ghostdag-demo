use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStatus {
    Pending,
    Executed,
    Failed(String),
    Reverted,
}

/// Transaction structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub status: TxStatus,
}

impl Transaction {
    pub fn new(id: String, from: String, to: String, amount: u64, nonce: u64) -> Self {
        Self {
            id,
            from,
            to,
            amount,
            nonce,
            status: TxStatus::Pending,
        }
    }
}

/// Block color (GHOSTDAG protocol)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockColor {
    Blue, // On the main chain
    Red,  // Not on the main chain
}

/// Block structure
#[derive(Debug, Clone)]
pub struct Block {
    pub hash: String,
    pub parent_hashes: Vec<String>, // Can have multiple parent blocks in DAG
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
    pub height: u64,
    pub color: BlockColor,
    pub weight: u64, // Cumulative weight
}

impl Block {
    pub fn new(
        hash: String,
        parent_hashes: Vec<String>,
        transactions: Vec<Transaction>,
        timestamp: u64,
    ) -> Self {
        Self {
            hash,
            parent_hashes,
            transactions,
            timestamp,
            height: 0,
            color: BlockColor::Blue,
            weight: 0,
        }
    }

    pub fn genesis() -> Self {
        Self {
            hash: "genesis".to_string(),
            parent_hashes: vec![],
            transactions: vec![],
            timestamp: 0,
            height: 0,
            color: BlockColor::Blue,
            weight: 1,
        }
    }
}

/// Account state
#[derive(Debug, Clone)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

impl Account {
    pub fn new(address: String, balance: u64) -> Self {
        Self {
            address,
            balance,
            nonce: 0,
        }
    }
}

/// BlockDAG - DAG-based blockchain
pub struct BlockDAG {
    blocks: HashMap<String, Block>,
    children_mapping: HashMap<String, HashSet<String>>, // Child block mapping
    accounts: HashMap<String, Account>,
    k: usize, // GHOSTDAG parameter, controls anticone size
}

impl BlockDAG {
    pub fn new(k: usize) -> Self {
        let mut dag = Self {
            blocks: HashMap::new(),
            children_mapping: HashMap::new(),
            accounts: HashMap::new(),
            k,
        };

        // Add genesis block
        let genesis = Block::genesis();
        dag.blocks.insert("genesis".to_string(), genesis);
        dag.children_mapping
            .insert("genesis".to_string(), HashSet::new());

        dag
    }

    /// Add account
    pub fn add_account(&mut self, address: String, balance: u64) {
        self.accounts
            .insert(address.clone(), Account::new(address, balance));
    }

    /// Get account
    pub fn get_account(&self, address: &str) -> Option<&Account> {
        self.accounts.get(address)
    }

    /// Add block to DAG
    pub fn add_block(&mut self, block: Block) -> Result<(), String> {
        // Verify parent blocks exist
        for parent in &block.parent_hashes {
            if !self.blocks.contains_key(parent) {
                return Err(format!("Parent block '{}' does not exist", parent));
            }
        }

        let hash = block.hash.clone();

        // Update parent blocks' children list
        for parent in &block.parent_hashes {
            self.children_mapping
                .get_mut(parent)
                .unwrap()
                .insert(hash.clone());
        }

        // Add block
        self.blocks.insert(hash.clone(), block);
        self.children_mapping.insert(hash.clone(), HashSet::new());

        // Recalculate GHOSTDAG ordering
        self.update_ghostdag_ordering()?;

        Ok(())
    }

    /// GHOSTDAG algorithm: calculate blue block set and ordering
    fn update_ghostdag_ordering(&mut self) -> Result<(), String> {
        let ordered_blocks = self.ghostdag_sort()?;

        // Reset all block colors and weights
        for block in self.blocks.values_mut() {
            block.color = BlockColor::Red;
            block.weight = 0;
        }

        // Mark blue blocks and calculate weights in GHOSTDAG order
        let mut weight = 0u64;
        for hash in ordered_blocks {
            if let Some(block) = self.blocks.get_mut(&hash) {
                block.color = BlockColor::Blue;
                weight += 1;
                block.weight = weight;
            }
        }

        Ok(())
    }

    /// GHOSTDAG topological sort
    fn ghostdag_sort(&self) -> Result<Vec<String>, String> {
        let mut blue_set = HashSet::new();
        let mut ordered = Vec::new();

        // Calculate in-degrees (number of parents) for all blocks
        let mut in_degree = HashMap::new();
        for block in self.blocks.values() {
            in_degree.insert(block.hash.clone(), block.parent_hashes.len());
        }

        // Priority queue for topological sort
        // Stores (parent_count, Reverse(timestamp), hash)
        // Prioritizes:
        // 1. More parents (higher complexity/merge blocks)
        // 2. Earlier timestamp (Reverse(timestamp))
        let mut heap = BinaryHeap::new();

        // Initialize heap with blocks that have 0 in-degree (genesis)
        if let Some(genesis) = self.blocks.get("genesis") {
            heap.push((
                genesis.parent_hashes.len(),
                Reverse(genesis.timestamp),
                genesis.hash.clone(),
            ));
        }

        while let Some((_, _, current_hash)) = heap.pop() {
            // Determine if block is blue
            // Genesis is always blue
            let is_blue = if current_hash == "genesis" {
                true
            } else {
                self.is_blue_candidate(&current_hash, &blue_set)
            };

            if is_blue {
                blue_set.insert(current_hash.clone());
                ordered.push(current_hash.clone());
            }

            // Process children
            if let Some(children) = self.children_mapping.get(&current_hash) {
                for child_hash in children {
                    if let Some(degree) = in_degree.get_mut(child_hash) {
                        *degree -= 1;
                        if *degree == 0 {
                            // All parents processed, add to heap
                            if let Some(child_block) = self.blocks.get(child_hash) {
                                heap.push((
                                    child_block.parent_hashes.len(),
                                    Reverse(child_block.timestamp),
                                    child_hash.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(ordered)
    }

    /// Determine if candidate block should be blue
    fn is_blue_candidate(&self, candidate: &str, blue_set: &HashSet<String>) -> bool {
        let _block = match self.blocks.get(candidate) {
            Some(b) => b,
            None => return false,
        };

        // Check number of blue blocks in anticone
        let anticone = self.get_anticone(candidate, blue_set);
        let blue_anticone_size = anticone
            .iter()
            .filter(|hash| blue_set.contains(*hash))
            .count();

        // If blue anticone size doesn't exceed k, block can be blue
        blue_anticone_size <= self.k
    }

    /// Get anticone (blocks that are neither ancestors nor descendants)
    fn get_anticone(&self, block_hash: &str, reference_set: &HashSet<String>) -> HashSet<String> {
        let mut anticone = HashSet::new();
        let ancestors = self.get_ancestors(block_hash);
        let descendants = self.get_descendants(block_hash);

        for hash in reference_set {
            if hash != block_hash && !ancestors.contains(hash) && !descendants.contains(hash) {
                anticone.insert(hash.clone());
            }
        }

        anticone
    }

    /// Get all ancestor blocks
    fn get_ancestors(&self, block_hash: &str) -> HashSet<String> {
        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(block) = self.blocks.get(block_hash) {
            for parent in &block.parent_hashes {
                queue.push_back(parent.clone());
                ancestors.insert(parent.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(block) = self.blocks.get(&current) {
                for parent in &block.parent_hashes {
                    if ancestors.insert(parent.clone()) {
                        queue.push_back(parent.clone());
                    }
                }
            }
        }

        ancestors
    }

    /// Get all descendant blocks
    fn get_descendants(&self, block_hash: &str) -> HashSet<String> {
        let mut descendants = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(children) = self.children_mapping.get(block_hash) {
            for child in children {
                queue.push_back(child.clone());
                descendants.insert(child.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.children_mapping.get(&current) {
                for child in children {
                    if descendants.insert(child.clone()) {
                        queue.push_back(child.clone());
                    }
                }
            }
        }

        descendants
    }

    /// Get blue blocks ordered by weight
    pub fn get_ordered_blue_blocks(&self) -> Vec<&Block> {
        let mut blue_blocks: Vec<&Block> = self
            .blocks
            .values()
            .filter(|b| b.color == BlockColor::Blue)
            .collect();

        blue_blocks.sort_by_key(|b| b.weight);
        blue_blocks
    }

    /// Execute transaction
    fn execute_transaction(&mut self, tx: &mut Transaction) -> Result<(), String> {
        // Check sender account
        let sender = self
            .accounts
            .get_mut(&tx.from)
            .ok_or_else(|| format!("Sender account '{}' does not exist", tx.from))?;

        // Verify nonce
        if tx.nonce != sender.nonce {
            tx.status = TxStatus::Failed(format!(
                "Invalid nonce: expected {}, got {}",
                sender.nonce, tx.nonce
            ));
            return Err(tx.status.clone().to_string());
        }

        // Verify balance
        if sender.balance < tx.amount {
            tx.status = TxStatus::Failed(format!(
                "Insufficient balance: has {}, needs {}",
                sender.balance, tx.amount
            ));
            return Err(tx.status.clone().to_string());
        }

        // Execute transfer
        sender.balance -= tx.amount;
        sender.nonce += 1;

        // Receiver account
        if let Some(receiver) = self.accounts.get_mut(&tx.to) {
            receiver.balance += tx.amount;
        } else {
            // If receiver doesn't exist, create new account
            self.accounts
                .insert(tx.to.clone(), Account::new(tx.to.clone(), tx.amount));
        }

        tx.status = TxStatus::Executed;
        Ok(())
    }

    /// Execute all transactions in blue blocks
    pub fn execute_blue_chain(&mut self) -> Result<(), String> {
        let blue_blocks = self.get_ordered_blue_blocks();
        let block_hashes: Vec<String> = blue_blocks.iter().map(|b| b.hash.clone()).collect();

        for hash in block_hashes {
            // Clone transaction list first
            let transactions = if let Some(block) = self.blocks.get(&hash) {
                block.transactions.clone()
            } else {
                continue;
            };

            // Execute transactions and collect results
            let mut results = Vec::new();
            for mut tx in transactions {
                let result = self.execute_transaction(&mut tx);
                results.push((tx, result));
            }

            // Update transaction status in block
            if let Some(block) = self.blocks.get_mut(&hash) {
                for (i, (tx, _)) in results.iter().enumerate() {
                    if i < block.transactions.len() {
                        block.transactions[i] = tx.clone();
                    }
                }
            }
        }

        Ok(())
    }

    /// Revert transaction
    fn revert_transaction(&mut self, tx: &mut Transaction) -> Result<(), String> {
        if tx.status != TxStatus::Executed {
            return Err("Transaction was not executed".to_string());
        }

        // Restore sender balance and nonce
        if let Some(sender) = self.accounts.get_mut(&tx.from) {
            sender.balance += tx.amount;
            sender.nonce = sender.nonce.saturating_sub(1);
        }

        // Deduct receiver balance
        if let Some(receiver) = self.accounts.get_mut(&tx.to) {
            if receiver.balance >= tx.amount {
                receiver.balance -= tx.amount;
            }
        }

        tx.status = TxStatus::Reverted;
        Ok(())
    }

    /// Revert all transactions in specified block
    pub fn revert_block(&mut self, block_hash: &str) -> Result<(), String> {
        // Clone transaction list first
        let transactions = {
            let block = self
                .blocks
                .get(block_hash)
                .ok_or_else(|| format!("Block '{}' does not exist", block_hash))?;
            block.transactions.clone()
        };

        // Execute transaction rollback in reverse order
        let tx_count = transactions.len();
        let mut reverted_txs = Vec::new();

        for i in (0..tx_count).rev() {
            let mut tx = transactions[i].clone();
            self.revert_transaction(&mut tx)?;
            reverted_txs.push((i, tx));
        }

        // Update transaction status in block
        let block = self.blocks.get_mut(block_hash).unwrap();
        for (i, tx) in reverted_txs {
            block.transactions[i] = tx;
        }

        Ok(())
    }

    /// Get block
    pub fn get_block(&self, hash: &str) -> Option<&Block> {
        self.blocks.get(hash)
    }

    /// Get all blocks
    pub fn get_all_blocks(&self) -> Vec<&Block> {
        self.blocks.values().collect()
    }
}

impl TxStatus {
    fn to_string(&self) -> String {
        match self {
            TxStatus::Pending => "Pending".to_string(),
            TxStatus::Executed => "Executed".to_string(),
            TxStatus::Failed(reason) => format!("Failed: {}", reason),
            TxStatus::Reverted => "Reverted".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_ghostdag_linear_chain() {
        // Test GHOSTDAG with a simple linear chain (no forks)
        // DAG structure:
        //   genesis -> b1 -> b2 -> b3
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
        // DAG structure:
        //      genesis
        //       /   \
        //      b1   b2
        // When two blocks compete, both may still be blue if their anticone relationship allows
        let mut dag = BlockDAG::new(1);

        // Create a fork: genesis -> b1 and genesis -> b2
        let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
        let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);

        dag.add_block(b1).unwrap();
        dag.add_block(b2).unwrap();

        // With k=1, the first block (b1) should be blue
        assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);

        // b2's anticone contains b1 (1 blue block), which equals k=1, so b2 can also be blue
        assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);

        // Verify weights
        // Weight doesn't represent "importance" or "validity" - both blocks are equally blue/valid.
        // Weight represents their position in the canonical ordering that's needed for deterministic transaction execution.
        // The algorithm breaks the tie using timestamp: b1 has timestamp 100, b2 has timestamp 200
        // b1 gets processed first → weight 2
        // b2 gets processed second → weight 3
        assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
        assert_eq!(dag.get_block("b1").unwrap().weight, 2);
        assert_eq!(dag.get_block("b2").unwrap().weight, 3);
    }

    #[test]
    fn test_ghostdag_simple_fork_k3() {
        // Test GHOSTDAG with k=3 (allows parallel blocks)
        // DAG structure:
        //      genesis
        //       /   \
        //      b1   b2
        // Both competing blocks should be blue
        let mut dag = BlockDAG::new(3);

        let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
        let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);

        dag.add_block(b1).unwrap();
        dag.add_block(b2).unwrap();

        // With k=3, both should be blue (anticone size is 1, which is <= k)
        assert_eq!(dag.get_block("b1").unwrap().color, BlockColor::Blue);
        assert_eq!(dag.get_block("b2").unwrap().color, BlockColor::Blue);

        // Verify specific weights based on timestamp ordering
        assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
        assert_eq!(dag.get_block("b1").unwrap().weight, 2);
        assert_eq!(dag.get_block("b2").unwrap().weight, 3);
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
        let blue_count = [
            dag.get_block("b1").unwrap(),
            dag.get_block("b2").unwrap(),
            dag.get_block("b3").unwrap(),
        ]
        .iter()
        .filter(|b| b.color == BlockColor::Blue)
        .count();

        // With k=2: b1, b2, b3 all have anticone size 2 (each other)
        // All 3 should be blue since anticone size (2) <= k (2)
        assert_eq!(blue_count, 3);

        // b4 should be blue (it references all parents)
        assert_eq!(dag.get_block("b4").unwrap().color, BlockColor::Blue);

        // Verify specific weights:
        // 1. Genesis (1)
        // 2. b1 (ts=100) -> 2
        // 3. b2 (ts=200) -> 3
        // 4. b3 (ts=300) -> 4
        // 5. b4 (child of b1,b2,b3) -> 5
        assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
        assert_eq!(dag.get_block("b1").unwrap().weight, 2);
        assert_eq!(dag.get_block("b2").unwrap().weight, 3);
        assert_eq!(dag.get_block("b3").unwrap().weight, 4);
        assert_eq!(dag.get_block("b4").unwrap().weight, 5);
    }
    #[test]
    fn test_ghostdag_weight_ordering() {
        // Test that weights are properly ordered in the blue chain
        // DAG structure:
        //      genesis
        //       /   \
        //      b1   b2
        //       |    \
        //      b3     |
        //        \   /
        //         b4
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

        // revert sequence of b2 and b1, but they will be ordered by timestamp later
        dag.add_block(b2).unwrap();
        dag.add_block(b1).unwrap();

        dag.add_block(b3).unwrap();
        dag.add_block(b4).unwrap();

        // Get ordered blue blocks
        let blue_blocks = dag.get_ordered_blue_blocks();

        // Verify specific weights showing topological then timestamp ordering
        assert_eq!(blue_blocks.len(), 5);
        assert_eq!(blue_blocks[0].hash, "genesis");
        assert_eq!(blue_blocks[0].weight, 1);

        // b1 (ts=100) -> weight 2
        assert_eq!(blue_blocks[1].hash, "b1");
        assert_eq!(blue_blocks[1].weight, 2);

        // b2 (ts=200) -> weight 3
        assert_eq!(blue_blocks[2].hash, "b2");
        assert_eq!(blue_blocks[2].weight, 3);

        // b3 (child of b1, ts=300) -> weight 4
        assert_eq!(blue_blocks[3].hash, "b3");
        assert_eq!(blue_blocks[3].weight, 4);

        // b4 (child of b1,b2,b3, ts=400) -> weight 5
        assert_eq!(blue_blocks[4].hash, "b4");
        assert_eq!(blue_blocks[4].weight, 5);
    }

    #[test]
    fn test_ghostdag_timestamp_ordering() {
        // Test that earlier timestamp blocks are preferred when anticone size allows
        // DAG structure:
        //        genesis
        //        /  |
        //      b2  b3  b1  (timestamps: b2=100, b3=300, b1=500)
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

        // Verify timestamp ordering: b2(100) < b3(300) < b1(500)
        assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
        assert_eq!(dag.get_block("b2").unwrap().weight, 2);
        assert_eq!(dag.get_block("b3").unwrap().weight, 3);
        assert_eq!(dag.get_block("b1").unwrap().weight, 4);
    }

    #[test]
    fn test_ghostdag_anticone_calculation() {
        // Test that anticone is correctly calculated
        // DAG structure:
        //      genesis
        //       /
        //      b1   b2  (b1 and b2 are in each other's anticone)
        //       \   /
        //        b3    (b3 is descendant of both, not in their anticone)
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

        // Verify weights:
        // 1. Genesis (1)
        // 2. b1 (ts=100) -> 2
        // 3. b2 (ts=200) -> 3
        // 4. b3 (child of b1,b2) -> 4
        assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
        assert_eq!(dag.get_block("b1").unwrap().weight, 2);
        assert_eq!(dag.get_block("b2").unwrap().weight, 3);
        assert_eq!(dag.get_block("b3").unwrap().weight, 4);
    }

    #[test]
    fn test_ghostdag_red_block_exclusion() {
        // Test that red blocks don't contribute to weight and are properly excluded
        // DAG structure:
        //      genesis
        //       /
        //      b1   b2  (with k=0, only one can be blue)
        let mut dag = BlockDAG::new(0); // k=0 means only one chain can be blue

        let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
        let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);

        dag.add_block(b1).unwrap();
        dag.add_block(b2).unwrap();

        // With k=0, only one fork can be blue
        let blue_count = [dag.get_block("b1").unwrap(), dag.get_block("b2").unwrap()]
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

        assert_eq!(red_block.weight, 0, "Red block should have weight 0");
    }

    #[test]
    fn test_ghostdag_multiple_parents_ordering() {
        // Test ordering when a block has multiple parents
        // DAG structure:
        //        genesis
        //        /  |
        //      b1  b2  b3
        //        \  |  /
        //          b4    (b4 has 3 parents, higher depth)
        let mut dag = BlockDAG::new(5);

        let b1 = Block::new("b1".to_string(), vec!["genesis".to_string()], vec![], 100);
        let b2 = Block::new("b2".to_string(), vec!["genesis".to_string()], vec![], 200);
        let b3 = Block::new("b3".to_string(), vec!["genesis".to_string()], vec![], 300);

        // b4 has more parents (3 vs 1)
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

        // Verify weights:
        // 1. Genesis (1)
        // 2. b1 (ts=100) -> 2
        // 3. b2 (ts=200) -> 3
        // 4. b3 (ts=300) -> 4
        // 5. b4 (child of b1,b2,b3) -> 5
        assert_eq!(dag.get_block("genesis").unwrap().weight, 1);
        assert_eq!(dag.get_block("b1").unwrap().weight, 2);
        assert_eq!(dag.get_block("b2").unwrap().weight, 3);
        assert_eq!(dag.get_block("b3").unwrap().weight, 4);
        assert_eq!(dag.get_block("b4").unwrap().weight, 5);
    }
}
