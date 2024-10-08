use {
    anchor_lang::{prelude::borsh, Discriminator},
    anchor_mpt_demo::{instruction::ValidateMpt, ID},
    dotenv::dotenv,
    eyre::Result,
    solana_rpc_client::rpc_client,
    solana_sdk::{
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction,
    },
    std::{env, str::FromStr},
};

mod args;
use args::MptParams;

// const RPC_ADDR: &str = "https://api.devnet.solana.com";
const RPC_ADDR: &str = "http://127.0.0.1:8899";

fn main() -> Result<()> {
    dotenv().ok();
    let proof = MptParams::load();
    let program_id = Pubkey::from_str(ID.to_string().as_str()).unwrap();

    let private_key_str = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not found in .env file");
    let private_key_bytes: Vec<u8> = private_key_str
        .trim_matches(|c: char| !c.is_digit(10) && c != ',')
        .split(',') // 以逗号分割字符串
        .filter_map(|s| s.parse::<u8>().ok())
        .collect();

    let signer = Keypair::from_bytes(&private_key_bytes).unwrap();
    let client = rpc_client::RpcClient::new(RPC_ADDR);

    let ixs = vec![verify_instruction(program_id, &proof)];
    let latest_blockhash = client.get_latest_blockhash().unwrap();

    println!("latest_blockhash: {}", latest_blockhash);

    let sig = client
        .send_and_confirm_transaction(&transaction::Transaction::new_signed_with_payer(
            &ixs,
            Some(&signer.pubkey()),
            &[&signer],
            latest_blockhash,
        ))
        .unwrap();

    println!("tx: {}", sig);

    Ok(())
}

pub fn verify_instruction(program_id: Pubkey, params: &MptParams) -> Instruction {
    let instruction_data = ValidateMpt {
        proof: params.proof(),
        key: params.key.clone(),
        root_hash: params.root.clone(),
    };
    let mut data = borsh::to_vec(&instruction_data).unwrap();

    data.splice(0..0, ValidateMpt::DISCRIMINATOR.iter().cloned());

    Instruction {
        program_id,
        accounts: vec![],
        data,
    }
}
