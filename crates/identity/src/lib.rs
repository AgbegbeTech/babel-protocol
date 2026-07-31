use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, SigningKey};
use protocol_core::{ProtocolError, ProtocolEvent};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PersonIdentity {
    pub id: String,
    pub display_name: String,
    pub preferred_languages: Vec<String>,
    pub public_identity_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceIdentity {
    pub id: String,
    pub person_id: String,
    pub display_name: String,
    pub public_key: String,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceKeyMaterial {
    pub device_id: String,
    pub public_key: String,
    pub private_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("invalid private key")]
    InvalidPrivateKey,
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("invalid base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
}

pub fn generate_device_key_material() -> DeviceKeyMaterial {
    let signing_key = SigningKey::generate(&mut OsRng);
    DeviceKeyMaterial {
        device_id: format!("device-{}", Uuid::new_v4()),
        public_key: STANDARD.encode(signing_key.verifying_key().to_bytes()),
        private_key: STANDARD.encode(signing_key.to_bytes()),
    }
}

pub fn sign_event_with_private_key(
    event: &mut ProtocolEvent,
    private_key_b64: &str,
) -> Result<(), IdentityError> {
    let bytes = STANDARD.decode(private_key_b64)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| IdentityError::InvalidPrivateKey)?;
    let signing_key = SigningKey::from_bytes(&bytes);
    let signature = signing_key.sign(&event.signing_bytes()?);
    event.signature = STANDARD.encode(signature.to_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use protocol_core::{EventScope, ProtocolEvent};
    use serde_json::json;

    use super::{generate_device_key_material, sign_event_with_private_key};

    #[test]
    fn generated_device_key_signs_and_verifies() {
        let keys = generate_device_key_material();
        let mut event = ProtocolEvent::new(
            "babel.message.created/1",
            "did:babel:test",
            &keys.device_id,
            None,
            1,
            EventScope::Room,
            json!({"original_text":"Hello"}),
        );
        sign_event_with_private_key(&mut event, &keys.private_key).unwrap();
        assert!(event.verify_with_public_key_b64(&keys.public_key).is_ok());
    }
}
