use {
    num_bigint::BigUint,
    serde::{Deserialize, Serialize},
    std::fs,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct MptParams {
    pub root: String,
    pub key: String,
    pub proof: Vec<String>,
}

const MPT_PATH: &str = "cli/proof/mpt.json";

impl MptParams {
    pub fn load() -> Self {
        let contents = fs::read_to_string(MPT_PATH).unwrap();
        serde_json::from_str(&contents).unwrap()
    }

    pub fn key(&self) -> Vec<u8> {
        let big_int = BigUint::parse_bytes(self.key.as_bytes(), 16).unwrap();
        big_int.to_bytes_be()
    }

    pub fn root(&self) -> String {
        self.root.to_owned()
    }

    pub fn proof(&self) -> Vec<Vec<u8>> {
        let mut proofs = Vec::new();
        for p in self.proof.clone().into_iter() {
            let value = BigUint::parse_bytes(p.as_bytes(), 16).unwrap();
            proofs.push(value.to_bytes_be());
        }
        proofs
    }
}
