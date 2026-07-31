use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const PROTOCOL_ID: &str = "babel/1";
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventScope {
    Private,
    Room,
    Community,
    Federated,
    Public,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AttachmentReference {
    pub id: String,
    pub media_type: String,
    pub byte_length: u64,
    pub content_hash: String,
    pub storage_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProtocolEvent {
    pub protocol: String,
    pub id: String,
    pub schema: String,
    pub version: u32,
    pub author_id: String,
    pub device_id: String,
    pub room_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub client_sequence: u64,
    pub parent_ids: Vec<String>,
    pub scope: EventScope,
    pub expires_at: Option<DateTime<Utc>>,
    pub content: serde_json::Value,
    pub attachments: Vec<AttachmentReference>,
    pub signature: String,
}

#[derive(Serialize)]
struct ProtocolEventSigningPayload<'a> {
    protocol: &'a str,
    id: &'a str,
    schema: &'a str,
    version: u32,
    author_id: &'a str,
    device_id: &'a str,
    room_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    client_sequence: u64,
    parent_ids: &'a [String],
    scope: &'a EventScope,
    expires_at: Option<DateTime<Utc>>,
    content: &'a serde_json::Value,
    attachments: &'a [AttachmentReference],
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("unsupported protocol identifier: {0}")]
    UnsupportedProtocol(String),
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u32),
    #[error("schema must not be empty")]
    MissingSchema,
    #[error("event author, device, and id are required")]
    MissingIdentity,
    #[error("event signature is missing")]
    MissingSignature,
    #[error("invalid base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("replay or stale sequence for device {device_id}: {sequence}")]
    ReplayDetected { device_id: String, sequence: u64 },
}

impl ProtocolEvent {
    pub fn new(
        schema: impl Into<String>,
        author_id: impl Into<String>,
        device_id: impl Into<String>,
        room_id: Option<Uuid>,
        client_sequence: u64,
        scope: EventScope,
        content: serde_json::Value,
    ) -> Self {
        Self {
            protocol: PROTOCOL_ID.to_string(),
            id: Uuid::new_v4().to_string(),
            schema: schema.into(),
            version: PROTOCOL_VERSION,
            author_id: author_id.into(),
            device_id: device_id.into(),
            room_id,
            created_at: Utc::now(),
            client_sequence,
            parent_ids: Vec::new(),
            scope,
            expires_at: None,
            content,
            attachments: Vec::new(),
            signature: String::new(),
        }
    }

    pub fn validate_version(&self) -> Result<(), ProtocolError> {
        if self.protocol != PROTOCOL_ID {
            return Err(ProtocolError::UnsupportedProtocol(self.protocol.clone()));
        }
        if self.version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(self.version));
        }
        if self.schema.trim().is_empty() {
            return Err(ProtocolError::MissingSchema);
        }
        if self.id.trim().is_empty()
            || self.author_id.trim().is_empty()
            || self.device_id.trim().is_empty()
        {
            return Err(ProtocolError::MissingIdentity);
        }
        Ok(())
    }

    pub fn signing_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        self.validate_version()?;
        let payload = ProtocolEventSigningPayload {
            protocol: &self.protocol,
            id: &self.id,
            schema: &self.schema,
            version: self.version,
            author_id: &self.author_id,
            device_id: &self.device_id,
            room_id: self.room_id,
            created_at: self.created_at,
            client_sequence: self.client_sequence,
            parent_ids: &self.parent_ids,
            scope: &self.scope,
            expires_at: self.expires_at,
            content: &self.content,
            attachments: &self.attachments,
        };
        Ok(serde_json::to_vec(&payload)?)
    }

    pub fn content_hash(&self) -> Result<String, ProtocolError> {
        let digest = Sha256::digest(self.signing_bytes()?);
        Ok(hex::encode(digest))
    }

    pub fn verify_with_public_key_b64(&self, public_key_b64: &str) -> Result<(), ProtocolError> {
        if self.signature.trim().is_empty() {
            return Err(ProtocolError::MissingSignature);
        }

        let public_key_bytes = STANDARD.decode(public_key_b64)?;
        let public_key_bytes: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| ProtocolError::InvalidPublicKey)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|_| ProtocolError::InvalidPublicKey)?;

        let signature_bytes = STANDARD.decode(&self.signature)?;
        let signature =
            Signature::from_slice(&signature_bytes).map_err(|_| ProtocolError::InvalidSignature)?;
        verifying_key
            .verify(&self.signing_bytes()?, &signature)
            .map_err(|_| ProtocolError::InvalidSignature)
    }
}

#[derive(Default)]
pub struct ReplayProtector {
    last_sequence_by_device: HashMap<String, u64>,
}

impl ReplayProtector {
    pub fn accept(&mut self, event: &ProtocolEvent) -> Result<(), ProtocolError> {
        let last = self
            .last_sequence_by_device
            .get(&event.device_id)
            .copied()
            .unwrap_or(0);
        if event.client_sequence <= last {
            return Err(ProtocolError::ReplayDetected {
                device_id: event.device_id.clone(),
                sequence: event.client_sequence,
            });
        }
        self.last_sequence_by_device
            .insert(event.device_id.clone(), event.client_sequence);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use ed25519_dalek::{Signer, SigningKey};
    use rand_core::OsRng;
    use serde_json::json;

    use super::{EventScope, ProtocolEvent, ReplayProtector};

    #[test]
    fn verifies_signed_event_and_detects_tampering() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
        let mut event = ProtocolEvent::new(
            "babel.message.created/1",
            "did:babel:amara",
            "device-amara",
            None,
            1,
            EventScope::Room,
            json!({"original_text":"Hello"}),
        );

        let signature = signing_key.sign(&event.signing_bytes().unwrap());
        event.signature = STANDARD.encode(signature.to_bytes());
        assert!(event.verify_with_public_key_b64(&public_key).is_ok());

        event.content = json!({"original_text":"Changed"});
        assert!(event.verify_with_public_key_b64(&public_key).is_err());
    }

    #[test]
    fn signature_is_excluded_from_content_hash() {
        let mut event = ProtocolEvent::new(
            "babel.message.created/1",
            "did:babel:amara",
            "device-amara",
            None,
            1,
            EventScope::Room,
            json!({"original_text":"Hello"}),
        );
        let first_hash = event.content_hash().unwrap();
        event.signature = "different signature bytes".to_string();
        assert_eq!(first_hash, event.content_hash().unwrap());
    }

    #[test]
    fn replay_protection_requires_increasing_device_sequence() {
        let mut replay = ReplayProtector::default();
        let event = ProtocolEvent::new(
            "babel.message.created/1",
            "did:babel:amara",
            "device-amara",
            None,
            4,
            EventScope::Room,
            json!({}),
        );
        assert!(replay.accept(&event).is_ok());
        assert!(replay.accept(&event).is_err());
    }
}
