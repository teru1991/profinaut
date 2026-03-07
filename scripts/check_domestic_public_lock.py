#!/usr/bin/env python3
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
inv = json.loads((root / 'ucel/coverage_v2/domestic_public/jp_public_inventory.json').read_text())
lock = json.loads((root / 'ucel/coverage_v2/domestic_public/jp_public_inventory.lock.json').read_text())

stable = sorted(f"{e['venue']}|{e['api_kind']}|{e['public_id']}" for e in inv['entries'])
assert stable == lock['stable_identifiers'], 'stable identifiers mismatch'
assert inv['venues'] == lock['venues'], 'venues mismatch'
print('domestic_public lock check: OK')
