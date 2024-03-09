use std::fmt::Debug;

use rand::Rng;

const MAX_NETWORK_START_BLOCK: u64 = 15299;
// since we assign 4 ips per address block, 60 * 4 = 240, leaving the last 15
// ips open.
const GROUPS_IN_LAST_BLOCK: u64 = 60;

pub trait ProvideIdentifier: Debug + Send + Sync {
    fn provide_identifier(&self) -> VmIdentifier;
}

#[derive(Debug, Default)]
pub struct VmIdentifierFactory;

impl ProvideIdentifier for VmIdentifierFactory {
    fn provide_identifier(&self) -> VmIdentifier {
        VmIdentifier::default()
    }
}

const ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

#[derive(Debug)]
pub struct VmIdentifier {
    id: String,
    address_block: AddressBlock,
}

impl VmIdentifier {
    pub fn new(id: String, address_block: u64) -> VmIdentifier {
        Self {
            id,
            address_block: AddressBlock::from(address_block),
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn address_block(&self) -> &AddressBlock {
        &self.address_block
    }
}

impl Default for VmIdentifier {
    fn default() -> Self {
        let id = nanoid::nanoid!(9, &ALPHABET);
        let address_block = rand::thread_rng().gen_range(0..=MAX_NETWORK_START_BLOCK);
        Self::new(id, address_block)
    }
}

#[derive(Clone, Debug)]
pub struct AddressBlock {
    base_address: String,
    starting_ip: u64,
}

/// Computes an address block from a number. We give each network 4 ip
/// addresses. The first one is the veth device and the second one is the vpeer
/// device. The third ip is the NAT'ed ip for the microvm. The fourth IP is
/// empty for now, maybe we'll find a use for it later.
impl From<u64> for AddressBlock {
    fn from(value: u64) -> Self {
        let block3 = value / GROUPS_IN_LAST_BLOCK;
        let block4 = (value % GROUPS_IN_LAST_BLOCK) * 4 + 1;

        Self {
            base_address: format!("10.200.{block3}"),
            starting_ip: block4,
        }
    }
}

impl AddressBlock {
    pub fn get_ip(&self, index: impl Into<u64>) -> String {
        format!("{}.{}", self.base_address, self.starting_ip + index.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::sandbox::id::GROUPS_IN_LAST_BLOCK;

    use super::{AddressBlock, MAX_NETWORK_START_BLOCK};

    #[test]
    fn test_no_panics_for_network_range() {
        for block in 0..=MAX_NETWORK_START_BLOCK {
            let _ = AddressBlock::from(block);
        }
    }

    #[test]
    fn next_door_neighbors() {
        let block1 = AddressBlock::from(0);
        let block2 = AddressBlock::from(1);

        assert_eq!(
            block1.base_address, block2.base_address,
            "address block 0 & 1 should have the same base address"
        );
        assert_eq!(
            block1.starting_ip + 4,
            block2.starting_ip,
            "next door neighbor blocks should be separated by 4"
        );
    }

    #[test]
    fn wrap_around_blocks() {
        let block1 = AddressBlock::from(GROUPS_IN_LAST_BLOCK - 1);
        let block2 = AddressBlock::from(GROUPS_IN_LAST_BLOCK);

        assert_ne!(
            block1.base_address, block2.base_address,
            "base address should be different after we hit the last group in a block"
        );

        assert!(block2.starting_ip == 1)
    }
}
