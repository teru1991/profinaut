# fileenc Crypto Contract (Domain B / Step3)

## Purpose
Provide production-grade encrypted secrets at rest via `.enc` records, with:
- deterministic AAD binding to context (path/field/registry/scope/version_hint)
- tamper detection (AEAD tag)
- fail-closed behavior
- versioned record format (v1 fixed here)

## v1 Record
- JSON UTF-8 object (sorted keys), fields:
  - magic: "UCEL-FILEENC"
  - v: 1
  - kdf: scrypt(salt,n,r,p,dklen=32)
  - aead: aes-256-gcm(nonce=12 bytes)
  - aad.context: base64(canonical json of FileEncContext)
  - ct: base64(AESGCM.encrypt output)

## AAD Binding
AAD must equal canonical json of:
{path, field, registry_id, scope, version_hint}
If mismatch => reject.

## Fail-closed
Decrypt failures, tamper, wrong passphrase, malformed record:
- must raise categorized error
- must not emit plaintext in any message/log.

## Tooling
`scripts/fileenc_tool.py encrypt` creates `.enc` from `.json` using FILEENC_PASSPHRASE.
