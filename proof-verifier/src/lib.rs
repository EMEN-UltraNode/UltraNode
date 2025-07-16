use sha2::{Digest, Sha256};

/// Build the Merkle root from hashed leaves.
pub fn build_merkle_root(leaves: &[Vec<u8>]) -> Vec<u8> {
    if leaves.len() == 1 {
        return leaves[0].clone();
    }

    let mut next_layer = vec![];

    for pair in leaves.chunks(2) {
        let combined = if pair.len() == 2 {
            [pair[0].as_slice(), pair[1].as_slice()].concat()
        } else {
            [pair[0].as_slice(), pair[0].as_slice()].concat()
        };

        let hash = Sha256::digest(&combined).to_vec();
        next_layer.push(hash);
    }

    build_merkle_root(&next_layer)
}

/// Generate a Merkle proof path for a given leaf index.
pub fn generate_proof(leaves: &[Vec<u8>], index: usize) -> Vec<Vec<u8>> {
    let mut path = vec![];
    let mut layer = leaves.to_vec();
    let mut idx = index;

    while layer.len() > 1 {
        let mut next_layer = vec![];

        for pair in layer.chunks(2) {
            let combined = if pair.len() == 2 {
                [pair[0].as_slice(), pair[1].as_slice()].concat()
            } else {
                [pair[0].as_slice(), pair[0].as_slice()].concat()
            };

            let hash = Sha256::digest(&combined).to_vec();
            next_layer.push(hash);
        }

        let sibling = if idx % 2 == 0 {
            if idx + 1 < layer.len() {
                layer[idx + 1].clone()
            } else {
                layer[idx].clone()
            }
        } else {
            layer[idx - 1].clone()
        };

        path.push(sibling);
        idx /= 2;
        layer = next_layer;
    }

    path
}

/// Verify a Merkle proof against the root.
pub fn verify_merkle_proof(leaf: &[u8], root: &[u8], proof: &[Vec<u8>], mut index: usize) -> bool {
    let mut hash = Sha256::digest(leaf).to_vec();

    for sibling in proof {
        let combined = if index % 2 == 0 {
            [hash.as_slice(), sibling.as_slice()].concat()
        } else {
            [sibling.as_slice(), hash.as_slice()].concat()
        };

        hash = Sha256::digest(&combined).to_vec();
        index /= 2;
    }

    hash == root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_leaf_indices() {
        let data = vec!["a", "b", "c", "d"];
        let leaves: Vec<Vec<u8>> = data
            .iter()
            .map(|d| Sha256::digest(d.as_bytes()).to_vec())
            .collect();

        let root = build_merkle_root(&leaves);

        for i in 0..leaves.len() {
            let proof = generate_proof(&leaves, i);
            assert!(
                verify_merkle_proof(data[i].as_bytes(), &root, &proof, i),
                "Proof failed at index {}",
                i
            );
        }
    }

    #[test]
    fn test_invalid_proof() {
        let data = vec!["a", "b", "c", "d"];
        let leaves: Vec<Vec<u8>> = data
            .iter()
            .map(|d| Sha256::digest(d.as_bytes()).to_vec())
            .collect();

        let root = build_merkle_root(&leaves);
        let bad_proof = vec![leaves[2].clone(), leaves[3].clone()];

        assert!(
            !verify_merkle_proof(data[0].as_bytes(), &root, &bad_proof, 0),
            "Invalid proof should fail"
        );
    }

    #[test]
    fn test_odd_number_of_leaves() {
        let data = vec!["x", "y", "z"];
        let leaves: Vec<Vec<u8>> = data
            .iter()
            .map(|d| Sha256::digest(d.as_bytes()).to_vec())
            .collect();

        let root = build_merkle_root(&leaves);

        for i in 0..leaves.len() {
            let proof = generate_proof(&leaves, i);
            assert!(
                verify_merkle_proof(data[i].as_bytes(), &root, &proof, i),
                "Odd-leaf proof failed at index {}",
                i
            );
        }
    }

    #[test]
    fn test_single_leaf_merkle_tree() {
        let data = vec!["solo"];
        let leaves: Vec<Vec<u8>> = data
            .iter()
            .map(|d| Sha256::digest(d.as_bytes()).to_vec())
            .collect();

        let root = build_merkle_root(&leaves);
        let proof = generate_proof(&leaves, 0);

        assert!(
            verify_merkle_proof(data[0].as_bytes(), &root, &proof, 0),
            "Single-leaf Merkle tree proof failed"
        );
    }
}
