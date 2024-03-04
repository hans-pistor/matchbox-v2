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
    counter: u64,
}

impl VmIdentifier {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn counter(&self) -> u64 {
        self.counter
    }
}

impl Default for VmIdentifier {
    fn default() -> Self {
        let id = nanoid::nanoid!(9, &ALPHABET);
        let counter = VM_ID_COUNTER.fetch_add(2, Ordering::Relaxed);
        Self { counter, id }
    }
}
