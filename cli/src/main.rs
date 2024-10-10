use {
    anchor_lang::{prelude::borsh, Discriminator},
    anchor_mpt_demo::{
        instruction::{AppendProof, InitProof, ValidateMpt, ViewProof},
        InitProofParams, ID,
    },
    dotenv::dotenv,
    eyre::Result,
    solana_rpc_client::rpc_client,
    solana_sdk::{
        // compute_budget::ComputeBudgetInstruction,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_program,
        transaction,
    },
    std::{env, str::FromStr},
};

mod args;
use args::MptParams;

// const RPC_ADDR: &str = "https://api.devnet.solana.com";
const RPC_ADDR: &str = "http://127.0.0.1:8899";
const TX_MAX_SIZE: usize = 996;
// called `Result::unwrap()` on an `Err` value: Error { request: Some(SendTransaction), kind: RpcError(RpcResponseError { code: -32602, message: "base64 encoded solana_sdk::transaction::versioned::VersionedTransaction too large: 1784 bytes (max: encoded/raw 1644/1232)", data: Empty }) }
// 1120 1808
// 1100 1784
// 1080 1756
// 1040 1704
// 1010 1664
// 1000 1648
// 998 1648
fn main() -> Result<()> {
    dotenv().ok();
    let proof = MptParams::load();
    // println!("proof: {:?}", proof.proof());
    let program_id = Pubkey::from_str(ID.to_string().as_str()).unwrap();

    let private_key_str = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not found in .env file");
    let private_key_bytes: Vec<u8> = private_key_str
        .trim_matches(|c: char| !c.is_digit(10) && c != ',')
        .split(',') // 以逗号分割字符串
        .filter_map(|s| s.parse::<u8>().ok())
        .collect();

    let signer = Keypair::from_bytes(&private_key_bytes).unwrap();
    let client = rpc_client::RpcClient::new(RPC_ADDR);

    let mut len: usize = 4;
    for v in proof.proof() {
        len += 4 + v.len();
    }
    let hash_root = proof.root();
    let seeds: &[&[u8]] = &[b"MptProof", hash_root.as_slice()];

    let (mpt_account, _) = Pubkey::find_program_address(&seeds, &program_id);

    let init_ix =
        init_proof_instruction(program_id, signer.pubkey(), mpt_account, &proof, len as u32);
    send_transaction(&client, &signer, &vec![init_ix], "init_proof_instruction");

    let mut data = Vec::new();
    let mut remaining_length = TX_MAX_SIZE;
    let mut is_merge = false;
    for d in proof.proof() {
        let cur_size = d.len();
        let mut remaining_data = Vec::new();
        if remaining_length > 0 {
            if remaining_length > cur_size {
                data.push(d);
                remaining_length -= cur_size;
            } else {
                // let l = cur_size - remaining_length;
                data.push(d[..remaining_length].to_vec());
                remaining_data = d[remaining_length..].to_vec();
                remaining_length = 0;
            }
        }

        if remaining_length > TX_MAX_SIZE {
            //错误
            // return Err("错误");
            println!("错误----remaining_length :{}", remaining_length);
        }

        if remaining_length == 0 {
            // for (k, v) in data.iter().enumerate() {
            //     println!("k: {}, v: {}", k, to_hex_string(v));
            // }

            let ix = append_proof_instruction(program_id, mpt_account, data.clone(), is_merge);
            send_transaction(&client, &signer, &vec![ix], "append_proof ");

            data.clear();
            remaining_length = TX_MAX_SIZE;
            is_merge = false;
        }

        if remaining_data.len() > 0 {
            //todo 有可能 remaining_length 大于 TX_MAX_SIZE

            remaining_length -= remaining_data.len();
            data.push(remaining_data);
            is_merge = true;
        }
    }

    // 最后一个循环可能还有数据
    if data.len() > 0 {
        // for (k, v) in data.iter().enumerate() {
        //     println!("k: {}, v: {}", k, to_hex_string(v));
        // }
        let ix = append_proof_instruction(program_id, mpt_account, data.clone(), is_merge);
        send_transaction(&client, &signer, &vec![ix], "append_proof ");
    }

    // let ix = view_instruction(program_id, mpt_account);
    // send_transaction(&client, &signer, &vec![ix], "view ");

    // let uc_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(500_0000);
    let ix = verify_instruction(program_id, mpt_account, &proof);
    send_transaction(&client, &signer, &vec![ix], "verify");
    Ok(())
}

pub fn init_proof_instruction(
    program_id: Pubkey,
    payer: Pubkey,
    mpt_account: Pubkey,
    params: &MptParams,
    proof_size: u32,
) -> Instruction {
    // let
    let instruction_data = InitProof {
        params: InitProofParams {
            hash_root: params.root(),
            proof_size,
        },
    };
    let mut data = borsh::to_vec(&instruction_data).unwrap();
    let account_metas = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(mpt_account, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    data.splice(0..0, InitProof::DISCRIMINATOR.iter().cloned());

    Instruction {
        program_id,
        accounts: account_metas,
        data,
    }
}

pub fn append_proof_instruction(
    program_id: Pubkey,
    mpt_account: Pubkey,
    data: Vec<Vec<u8>>,
    is_merge: bool,
) -> Instruction {
    let instruction_data = AppendProof { data, is_merge };
    let mut data = borsh::to_vec(&instruction_data).unwrap();
    let account_metas = vec![AccountMeta::new(mpt_account, false)];

    data.splice(0..0, AppendProof::DISCRIMINATOR.iter().cloned());

    Instruction {
        program_id,
        accounts: account_metas,
        data,
    }
}

pub fn view_instruction(program_id: Pubkey, mpt_account: Pubkey) -> Instruction {
    let instruction_data = ViewProof {};
    let mut data = borsh::to_vec(&instruction_data).unwrap();

    let account_metas = vec![AccountMeta::new(mpt_account, false)];

    data.splice(0..0, ViewProof::DISCRIMINATOR.iter().cloned());

    Instruction {
        program_id,
        accounts: account_metas,
        data,
    }
}

pub fn verify_instruction(
    program_id: Pubkey,
    mpt_account: Pubkey,
    params: &MptParams,
) -> Instruction {
    let instruction_data = ValidateMpt { key: params.key() };
    let mut data = borsh::to_vec(&instruction_data).unwrap();

    let account_metas = vec![AccountMeta::new(mpt_account, false)];

    data.splice(0..0, ValidateMpt::DISCRIMINATOR.iter().cloned());

    Instruction {
        program_id,
        accounts: account_metas,
        data,
    }
}

fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b: &u8| format!("{:02x}", b)).collect()
}

fn send_transaction(
    client: &rpc_client::RpcClient,
    signer: &Keypair,
    ixs: &Vec<Instruction>,
    msg: &str,
) {
    let latest_blockhash = client.get_latest_blockhash().unwrap();
    let sig = client
        .send_and_confirm_transaction(&transaction::Transaction::new_signed_with_payer(
            ixs,
            Some(&signer.pubkey()),
            &[&signer],
            latest_blockhash,
        ))
        .unwrap();

    println!("{} tx: {}", msg, sig);
}
