use ed25519_dalek::ed25519::signature::Signer;
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use super::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxUnsigned {
    sender: Address,
    recipient: Address,
    amount: u64,
}

impl TxUnsigned {
    fn new(sender: Address, recipient: Address, amount: u64) -> Self {
        TxUnsigned {
            sender,
            recipient,
            amount,
        }
    }
}

// Подписанная транзакция с возможностью проверки её подлинности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tx {
    sender: Address,
    recipient: Address,
    amount: u64,
    signature: Signature,
}

impl Tx {
    // Для подписи транзакции пользователь должен передать свой приватный ключ
    // В реальных условиях, данная операция должна производиться на компьютере пользователя
    // Далее подписанная транзакция может передаваться по сети
    pub fn sign(sender: Address, recipient: Address, amount: u64, signinig_key: SigningKey) -> Tx {
        let unsigned = TxUnsigned::new(sender, recipient, amount);
        let serialized = serde_json::to_string(&unsigned).unwrap();
        let signature: Signature = signinig_key.sign(&serialized.into_bytes());

        Tx {
            sender,
            recipient,
            amount,
            signature,
        }
    }

    // Верификация подлинности транзакции
    pub fn is_valid(&self) -> bool {
        let verifying_key: VerifyingKey = self.sender;
        let unsigned = TxUnsigned::new(self.sender, self.recipient, self.amount);
        let serialized = serde_json::to_string(&unsigned).unwrap();

        verifying_key
            .verify(&serialized.into_bytes(), &self.signature)
            .is_ok()
    }
}
