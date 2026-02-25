# ADR-002: Use bootloader_api Crate for Boot Interface

## Status
Accepted

## Background
The kernel needs a bootloader to transition from firmware (BIOS/UEFI) to
64-bit long mode and provide a memory map. Three options were evaluated.

## Decision
Depend on `bootloader_api` (v0.11) for boot info types and the `entry_point!`
macro. The full `bootloader` crate is used only as a build tool (not a cargo
dependency) to produce bootable disk images.

## Alternatives
1. **Write a custom bootloader in assembly** — extremely time-consuming and
   error-prone; does not align with the project's focus on kernel internals.
2. **Use `bootloader` as a regular dependency** — fails to compile for
   `x86_64-unknown-none` because it requires `std` (serde, fatfs, etc.).
3. **Use UEFI-rs directly** — viable but more complex; less community tooling.

## Consequences
- Disk image creation requires a separate build step (handled by Makefile.toml).
- The kernel binary is a standard ELF that the bootloader wraps.
