# File Format Notes

## Encryption Format v1

Used by `crypto_engine`.

```text
offset  size  field
0       8     magic = "VLTCRPT1"
8       1     version = 1
9       1     algorithm id
10      16    argon2 salt
26      24    xchacha20 nonce
50      n     ciphertext with auth tag
```

Algorithm ids:

- `1`: XChaCha20-Poly1305
- `2`: AES-256-GCM reserved

Notes:

- current MVP implements id `1`
- full payload is authenticated by AEAD

## Append Stego Format v1

Used by `stego_engine` in append mode.

```text
carrier bytes
magic = "VLTSTEG1"
version = 1
name_len = u16 be
salt = 16 bytes
nonce = 24 bytes
encrypted_len = u64 be
file_name bytes
encrypted compressed payload
```

Pipeline:

```text
payload -> zlib compress -> xchacha20-poly1305 encrypt -> append to carrier
```

Notes:

- extract mode searches for the last occurrence of `VLTSTEG1`
- current MVP restores content to a caller-provided output path
