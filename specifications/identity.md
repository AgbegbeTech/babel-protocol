# Identity and Devices

Person identity and device identity are separate.

```text
Person
├── Babel identity
├── display name
├── preferred languages
├── public identity key
├── authorized devices
├── recovery placeholder
└── optional external identity links
```

Each device has its own keypair. Raw private keys are never sent to the server. Device revocation must be checked before accepting signed events.

Future interfaces include passkeys, WebAuthn, hardware-backed keys, recovery delegates, pseudonymous participation, and anonymous participation.
