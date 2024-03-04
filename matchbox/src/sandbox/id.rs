use std::sync::atomic::{AtomicU64, Ordering};

static VM_ID_COUNTER: AtomicU64 = AtomicU64::new(10);

const ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

#[derive(Debug)]
pub struct VmIdentifier {
    id: String,
    address_block: u64,
}

impl VmIdentifier {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn address_block(&self) -> u64 {
        self.address_block
    }
}

impl Default for VmIdentifier {
    fn default() -> Self {
        let id = nanoid::nanoid!(9, &ALPHABET);
        let address_block = VM_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self { address_block, id }
    }
}
