use protocol_core::{EventScope, ProtocolEvent, PROTOCOL_ID};
use serde_json::json;

fn main() {
    let event = ProtocolEvent::new(
        "babel.message.created/1",
        "did:babel:console",
        "device-console",
        None,
        1,
        EventScope::Private,
        json!({"note":"Babel node console is a developer inspection helper."}),
    );

    println!("{} console ready", PROTOCOL_ID);
    println!("sample_event_hash={}", event.content_hash().expect("hash"));
}
