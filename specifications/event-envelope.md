# Protocol Event Envelope

Protocol identifier: `babel/1`

DID pattern: `did:babel:*`

Canonical Rust structure:

```rust
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
```

Supported scopes:

```text
private
room
community
federated
public
```

The signature is excluded from the event hash. Verification covers protocol, schema, version, author, device, room, timestamp, client sequence, parent references, scope, expiry, content, and attachments.

Replay protection is per device and requires monotonically increasing client sequences.
