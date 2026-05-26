#!/usr/bin/env python3
"""
Compute SPEL IDL instruction discriminators.

Discriminator = SHA256("global:{instruction_name}")[0..8]

Run: python3 interfaces/compute_discriminators.py
Then paste the output arrays into lp0002.idl.json.
"""
import hashlib
import json
import sys

INSTRUCTIONS = [
    "create_multisig",
    "create_proposal",
    "submit_approval",
    "execute_threshold",
    "verify_and_execute",
    "verify_and_execute_bytes",
]

def compute_discriminator(name: str) -> list[int]:
    """SHA256('global:{name}')[..8] as list of 8 u8 ints."""
    preimage = f"global:{name}".encode("utf-8")
    digest = hashlib.sha256(preimage).digest()
    return list(digest[:8])

def main():
    print("// SPEL IDL discriminators for lp0002_private_multisig")
    print("// SHA256('global:{name}')[..8]\n")
    
    results = {}
    for name in INSTRUCTIONS:
        disc = compute_discriminator(name)
        results[name] = disc
        print(f"  {name}: {disc}")
    
    # Also output as JSON fragment for easy copy-paste
    print("\n// JSON fragment:")
    for name in INSTRUCTIONS:
        print(f'  "{name}": {json.dumps(compute_discriminator(name))},')

if __name__ == "__main__":
    main()
