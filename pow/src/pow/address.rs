use ed25519_dalek::VerifyingKey;

// Адресом пользователя выступает VerifyingKey
pub type Address = VerifyingKey;
