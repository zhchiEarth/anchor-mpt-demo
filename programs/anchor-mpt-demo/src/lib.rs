use {
    anchor_lang::prelude::*,
    // ethereum_types::H256,
    primitive_types_solana::H256,
    rlp::Rlp,
    std::str::FromStr,
};

mod errors;
mod nibbles;
mod node;
mod trie;

use trie::*;

declare_id!("9n2uAscxSNrotCE2PC1DpKgtUFu7iSWEiJpbN9ynqzHp");

#[program]
pub mod anchor_mpt_demo {
    use super::*;

    pub fn validate_mpt(
        ctx: Context<Initialize>,
        root_hash: String,
        key: String,
        proof: Vec<Vec<u8>>,
    ) -> Result<()> {
        let root_hash = H256::from_str(&root_hash).unwrap();

        let mut rlp_signed_tx = EthTrie::verify_proof(root_hash, key.as_bytes(), proof)
            .unwrap()
            .unwrap();
        rlp_signed_tx.remove(0);

        let rlp = Rlp::new(&rlp_signed_tx);
        println!("rlp: {:?}", rlp.at(0).unwrap().as_val::<u8>()); //  交易是否成功的状态  u8
        println!("rlp: {:?}", rlp.at(1).unwrap().as_val::<u64>()); //  effectiveGasPrice: u64

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

// #[derive(Debug)]
// pub struct LogInfo {
//     address: [u8; 20],
//     topics: Vec<[u8; 32]>,
//     data: Vec<u8>,
// }

// impl Decodable for LogInfo {
//     fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
//         let address = rlp.at(0).unwrap().data().unwrap().to_owned();
//         let address: [u8; 20] = match address.try_into() {
//             Ok(arr) => arr,
//             Err(e) => {
//                 // return Err("Failed to convert: {:?}", e);
//                 return Err(DecoderError::RlpExpectedToBeData);
//             }
//         };

//         let mut topic_list = Vec::new();
//         let topics = rlp.at(1).unwrap();
//         for topic in topics.iter() {
//             let value = topic.data().unwrap();
//             let t: [u8; 32] = match value.try_into() {
//                 Ok(arr) => arr,
//                 Err(e) => {
//                     // return Err("Failed to convert: {:?}", e);
//                     return Err(DecoderError::RlpExpectedToBeData);
//                 }
//             };
//             topic_list.push(t);
//         }
//         let data = rlp.at(2).unwrap().data().unwrap().to_owned();

//         Ok(LogInfo {
//             address,
//             topics: topic_list,
//             data,
//         })
//     }
// }
