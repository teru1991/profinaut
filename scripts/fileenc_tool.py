from __future__ import annotations

import argparse
import json
from pathlib import Path

from libs.safety_core.crypto.aead import Aes256Gcm
from libs.safety_core.crypto.fileenc_format import FileEncContext, b64e, dumps_record
from libs.safety_core.crypto.kdf import ScryptParams
from libs.safety_core.crypto.rot_passphrase import require_passphrase
from libs.safety_core.redaction import safe_str


def encrypt_json(*, src: Path, dst: Path, ctx: FileEncContext) -> None:
    obj = json.loads(src.read_text(encoding="utf-8"))
    if not isinstance(obj, dict):
        raise RuntimeError("input must be json object")

    params = ScryptParams.fresh()
    passphrase = require_passphrase()
    key = params.derive(passphrase)

    nonce = Aes256Gcm.fresh_nonce()
    aad = ctx.to_aad_bytes()
    a = Aes256Gcm(key=key)
    pt = json.dumps(obj, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")
    ct = a.encrypt(nonce=nonce, plaintext=pt, aad=aad)

    rec = {
        "magic": "UCEL-FILEENC",
        "v": 1,
        "kdf": {"name": "scrypt", "salt": b64e(params.salt), "n": params.n, "r": params.r, "p": params.p, "dklen": params.dklen},
        "aead": {"name": "aes-256-gcm", "nonce": b64e(nonce)},
        "aad": {"context": b64e(aad)},
        "ct": b64e(ct),
    }
    dst.write_bytes(dumps_record(rec))


def cmd_selftest() -> int:
    try:
        _ = require_passphrase()
        return 0
    except Exception as e:
        print("selftest failed:", safe_str(str(e)))
        return 2


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)

    enc = sub.add_parser("encrypt")
    enc.add_argument("--in", dest="src", required=True)
    enc.add_argument("--out", dest="dst", required=True)
    enc.add_argument("--path", required=True)
    enc.add_argument("--field", required=True)
    enc.add_argument("--registry-id", required=True)
    enc.add_argument("--scope", required=True)
    enc.add_argument("--version-hint", default=None)

    sub.add_parser("selftest")

    ns = ap.parse_args()
    try:
        if ns.cmd == "selftest":
            return cmd_selftest()
        if ns.cmd == "encrypt":
            ctx = FileEncContext(
                path=str(ns.path),
                field=str(ns.field),
                registry_id=str(ns.registry_id),
                scope=str(ns.scope),
                version_hint=str(ns.version_hint) if ns.version_hint else None,
            )
            encrypt_json(src=Path(ns.src), dst=Path(ns.dst), ctx=ctx)
            return 0
        raise RuntimeError("unknown cmd")
    except Exception as e:
        print("error:", safe_str(str(e)))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
