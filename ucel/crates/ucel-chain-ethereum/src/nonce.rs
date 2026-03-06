use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct NonceManager {
    // key = chain_id|account
    reserved: HashMap<String, u64>,
}

impl NonceManager {
    pub fn reserve(&mut self, chain_id: u64, account: &str, remote_pending_nonce: u64) -> u64 {
        let key = format!("{chain_id}|{account}");
        let local = self.reserved.get(&key).copied().unwrap_or(remote_pending_nonce);
        let next = local.max(remote_pending_nonce);
        self.reserved.insert(key, next + 1);
        next
    }

    pub fn rollback(&mut self, chain_id: u64, account: &str, nonce: u64) {
        let key = format!("{chain_id}|{account}");
        self.reserved.insert(key, nonce);
    }
}
