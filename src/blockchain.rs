use std::collections::{HashMap, HashSet, VecDeque};

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
    children: HashMap<String, HashSet<String>>, // Child block mapping
    accounts: HashMap<String, Account>,
    k: usize, // GHOSTDAG parameter, controls anticone size
}

impl BlockDAG {
    pub fn new(k: usize) -> Self {
        let mut dag = Self {
            blocks: HashMap::new(),
            children: HashMap::new(),
            accounts: HashMap::new(),
            k,
        };

        // Add genesis block
        let genesis = Block::genesis();
        dag.blocks.insert("genesis".to_string(), genesis);
        dag.children.insert("genesis".to_string(), HashSet::new());

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

        // Add block
        self.blocks.insert(hash.clone(), block);
        self.children.insert(hash.clone(), HashSet::new());

        // Update parent blocks' children list
        if let Some(blk) = self.blocks.get(&hash) {
            for parent in &blk.parent_hashes {
                self.children.get_mut(parent).unwrap().insert(hash.clone());
            }
        }

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

        // Start from genesis block
        blue_set.insert("genesis".to_string());
        ordered.push("genesis".to_string());

        // Get topological order of all blocks
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back("genesis".to_string());
        visited.insert("genesis".to_string());

        let mut candidates = Vec::new();

        // Collect all candidate blocks
        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.children.get(&current) {
                for child in children {
                    if !visited.contains(child) {
                        visited.insert(child.clone());
                        candidates.push(child.clone());
                        queue.push_back(child.clone());
                    }
                }
            }
        }

        // Sort candidate blocks by height and timestamp
        candidates.sort_by(|a, b| {
            let block_a = self.blocks.get(a).unwrap();
            let block_b = self.blocks.get(b).unwrap();

            // First sort by parent block count (more parents means deeper)
            let depth_a = block_a.parent_hashes.len();
            let depth_b = block_b.parent_hashes.len();

            if depth_a != depth_b {
                depth_b.cmp(&depth_a)
            } else {
                // Then by timestamp
                block_a.timestamp.cmp(&block_b.timestamp)
            }
        });

        // Use GHOSTDAG k-cluster algorithm
        for candidate in candidates {
            if self.is_blue_candidate(&candidate, &blue_set) {
                blue_set.insert(candidate.clone());
                ordered.push(candidate);
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

        if let Some(children) = self.children.get(block_hash) {
            for child in children {
                queue.push_back(child.clone());
                descendants.insert(child.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.children.get(&current) {
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
