use clap::{Parser, Subcommand};
use hex::{decode, encode};
use sha2::{Digest, Sha256};
use proof_verifier::{build_merkle_root, generate_proof, verify_merkle_proof};

/// Simple Merkle CLI
#[derive(Parser)]
#[command(name = "merkle-sim")]
#[command(about = "CLI to simulate Merkle tree proof generation + verification", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Merkle root + proof for a list of values
    Generate {
        /// Data entries (comma-separated)
        #[arg(short, long)]
        data: String,

        /// Index of leaf to generate proof for
        #[arg(short, long)]
        index: usize,
    },

    /// Verify a Merkle proof against a root
    Verify {
        /// Leaf hash (hex)
        #[arg(short, long)]
        leaf: String,

        /// Merkle root (hex)
        #[arg(short, long)]
        root: String,

        /// Proof path (comma-separated hex hashes)
        #[arg(short, long)]
        path: String,

        /// Index of the leaf
        #[arg(short, long)]
        index: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { data, index } => {
            let leaves: Vec<Vec<u8>> = data
                .split(',')
                .map(|s| Sha256::digest(s.as_bytes()).to_vec())
                .collect();

            let root = build_merkle_root(&leaves);
            let proof = generate_proof(&leaves, *index);

            println!("Merkle Root: {}", encode(&root));
            println!(
                "Proof Path: {}",
                proof.iter().map(|p| encode(p)).collect::<Vec<_>>().join(",")
            );
        }

        Commands::Verify {
            leaf,
            root,
            path,
            index,
        } => {
            let leaf_bytes = decode(leaf).unwrap();
            let root_bytes = decode(root).unwrap();
            let path_bytes: Vec<Vec<u8>> = path.split(',').map(|h| decode(h).unwrap()).collect();

            let valid = verify_merkle_proof(&leaf_bytes, &root_bytes, &path_bytes, *index);

            println!("Valid Proof? {}", valid);
        }
    }
}
