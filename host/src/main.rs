// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use alloy_primitives::{address, hex, Address, Signature as AlloySignature};
use alloy_signer::{Signer as AlloySigner, SignerSync};
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::{Context, Result};
use clap::Parser;
use erc20_methods::{ERC20_GUEST_ELF, ERC20_GUEST_ID};
// Signature,  - from ecsa..
use hex::decode;
use hex::FromHex;
use k256::ecdsa::{signature::Verifier, VerifyingKey};
use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
    EncodedPoint, SecretKey,
};
use risc0_steel::{config::ETH_SEPOLIA_CHAIN_SPEC, ethereum::EthEvmEnv, Contract, EvmBlockHeader};
use risc0_zkvm::{default_executor, default_prover, ExecutorEnv};
use std::fs::File;
use std::io::Write;
use tracing_subscriber::EnvFilter;

sol! {
    /// ERC-20 balance function signature.
    /// This must match the signature in the guest.
    interface IERC20 {
        function balanceOf(address account) external view returns (uint);
    }
}

/// Address of the deployed contract to call the function on (USDT contract on Sepolia).
const CONTRACT: Address = address!("aA8E23Fb1079EA71e0a56F48a2aA51851D8433D0");

/// Simple program to show the use of Ethereum contract data inside the guest.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// URL of the RPC endpoint
    /// https://sepolia.dev/ lists a few free for Sepolia to default to
    #[arg(
        short,
        long,
        env = "RPC_URL",
        default_value = "https://rpc2.sepolia.org/"
    )]
    rpc_url: String,

    /// Signing key of account to prove balance
    #[arg(
        short,
        long,
        env = "SIGNING_KEY",
        // Anvil's default 0th local testnet key
        // address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        default_value = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    )]
    signing_key: String,
}
fn main() -> Result<()> {
    // https://crates.io/crates/alloy-signer/0.1.2
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Should look like this!
    // let sig_hex = "79b84220ec3d5a9f1774390e72d6df4cbf61ab5516eb45c491be533f1980f34545b776c542f6206c9ae309bac345a4161e617c23a29f3d54569bf02b539da622";
    let sig_hex: String = std::env::args().nth(1).unwrap();
    let message: String = std::env::args().nth(2).unwrap();
    let sig_bytes = hex::decode(sig_hex.clone())?;
    let signature_secp = Signature::try_from(sig_bytes.as_slice())?;
    // TODO - understand how we can figure out the parity bit
    let signature = AlloySignature::from_signature_and_parity(signature_secp, true).unwrap();

    // Recover the signer from the message.
    // let recovered = signature.recover_address_from_msg(message)?;
    let recovered = signature.recover_address_from_msg(message.clone())?;
    println!("Recovered:: {}", recovered);

    /*
    let message_to_sign_hash = alloy_primitives::utils::keccak256(message.as_bytes());
    //let vk = sig2.recover_from_msg(message_to_sign_hash).unwrap();
    let vk: k256::ecdsa::VerifyingKey = sig2.recover_from_msg(message_to_sign_hash).unwrap();
    let vk_secp = k256::ecdsa::VerifyingKey::from_sec1_bytes(&vk.to_sec1_bytes()).unwrap();
    let matcha = vk_secp.verify(message_to_sign_hash.as_slice(), &signature);
    println!("Matcha: {:?}", matcha);
    */

    // let verify_key = VerifyingKey::from(&signing_key);
    // let val = vk.verify(message.as_bytes(), &sig2).unwrap();
    // et rtn=verify_key.verify(msg, &signature).is_ok();

    // Parse the command line arguments.
    // let rpc_url: String = std::env::args().nth(1).unwrap();
    let rpc_url: String = "https://rpc2.sepolia.org/".to_string();

    // ------------------------------------------------------------------------
    // Setting up: Steel view call
    // ------------------------------------------------------------------------

    // Create an EVM environment from an RPC endpoint and a block number. If no block number is
    // provided, the latest block is used.
    let mut env = EthEvmEnv::from_rpc(&rpc_url, None)?;
    //  The `with_chain_spec` method is used to specify the chain configuration.
    env = env.with_chain_spec(&ETH_SEPOLIA_CHAIN_SPEC);

    let commitment = env.block_commitment();

    // Function to call, implements the [SolCall] trait.
    let call = IERC20::balanceOfCall { account: recovered };

    // Preflight the call to prepare the input that is required to execute the function in
    // the guest without RPC access. It also returns the result of the call.
    let mut contract = Contract::preflight(CONTRACT, &mut env);
    let returns = contract.call_builder(&call).call()?;
    println!(
        "HOST: For block {} `{}` returns: {}",
        env.header().number(),
        IERC20::balanceOfCall::SIGNATURE,
        returns._0
    );

    // Finally, construct the input from the environment.
    let evm_input = env.into_input()?;

    // ------------------------------------------------------------------------
    // Takeoff: Execution & Proof generation
    // ------------------------------------------------------------------------

    // FIXME: no proof, execution only! When testing, we should never need to
    // make *real* proofs, in testing as in the zkVM we prove execution is
    // *exactly identical* to the result of that execution.
    // If there is a problem in the proof, it's a RISC Zero team issue more than likely, not yours.
    // If you *need* a proof, we can mock things using [risc0_zkvm::FakeReceipt]s in DEV_MODE.
    // let msg = "abcd";

    let session_info = {
        let env = ExecutorEnv::builder()
            .write(&sig_hex)
            .unwrap()
            .write(&message)
            .unwrap()
            .write(&recovered)
            .unwrap()
            .write(&evm_input)
            .unwrap()
            .build()
            .context("Failed to build exec env")?;

        // TODO - ask him?
        // let exec = default_executor();
        // exec.execute(env, ERC20_GUEST_ELF)
        //     .context("failed to run executor")?
        let prover = default_prover();

        let prove_info: risc0_zkvm::ProveInfo = prover.prove(env, ERC20_GUEST_ELF).unwrap();
        //////// Extra data...
        // extract the receipt.
        let receipt = prove_info.receipt;
        let receipt_inner_bytes_array = bincode::serialize(&receipt.inner).unwrap();

        // println!(
        //     "proof - Serialized bytes array (hex) INNER: {:?}\n",
        //     hex::encode(&receipt_inner_bytes_array)
        // );
        let file_proof = File::create("proof2.txt");
        let mut content = format!("0x{}", hex::encode(&receipt_inner_bytes_array));
        let _ = file_proof.unwrap().write_all(&content.as_bytes());
        let receipt_journal_bytes_array = bincode::serialize(&receipt.journal).unwrap();

        let file_pub = File::create("pub2.txt");
        content = format!("0x{}", hex::encode(&receipt_journal_bytes_array));
        let _ = file_pub.unwrap().write_all(&content.as_bytes());
        println!(
            "public inputs - Serialized bytes array (hex) JOURNAL: {:?}\n",
            hex::encode(&receipt_journal_bytes_array)
        );
        let mut image_id_hex = String::new();
        for &value in &ERC20_GUEST_ID {
            image_id_hex.push_str(&format!("{:08x}", value.to_be()));
        }

        // content = format!("0x{}", hex::encode(&receipt_journal_bytes_array));
        let file_vk = File::create("vk2.txt");
        // content = format!("0x{}", hex::encode(&image_id_hex));
        // let _ = file_vk.unwrap().write_all(&content.as_bytes());
        content = format!("0x{}", image_id_hex);
        let _ = file_vk.unwrap().write_all(&content.as_bytes());
        println!(
            "verification key - Serialized bytes array (hex) IMAGE_ID: {:?}\n",
            image_id_hex
        );
        let output: String = receipt.journal.decode().unwrap();
        println!("Output is: {}", output);
    };

    // ------------------------------------------------------------------------
    // Stick the landing: Test our assumptions and constrains hold
    // ------------------------------------------------------------------------

    // let (evm_committed_bytes, receipt_verifying_key, receipt_message): (
    //     Vec<u8>,
    //     EncodedPoint,
    //     Vec<u8>,
    // ) = session_info.journal.decode().unwrap();

    // let evm_committed_bytes: Vec<u8> = session_info.journal.decode().unwrap();

    // println!(
    //     "HOST: Verified the signature over message {:?} with key {}",
    //     std::str::from_utf8(&receipt_message[..]).unwrap(),
    //     receipt_verifying_key,
    // );

    // // The commitment in the journal should match.
    // let bytes = session_info.journal.as_ref();
    // assert!(evm_committed_bytes.starts_with(&commitment.abi_encode()));

    Ok(())
}
