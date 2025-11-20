# AIOS Bootloader

Secure bootloader with kernel verification and initramfs loading.

## Features

- UEFI and Multiboot support
- Kernel signature verification
- Initramfs loading
- Secure boot framework

## Building

```bash
cd boot/bootloader
cargo build --release
```

## Architecture

- `main.rs`: Bootloader entry point
- `uefi.rs`: UEFI boot support
- `multiboot.rs`: Multiboot protocol support
- `crypto.rs`: Cryptographic verification
- `initramfs.rs`: Initramfs loading

