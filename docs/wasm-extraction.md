# Verified Soroban Wasm Extraction

Phase 3 maps the contract specification embedded in Soroban Wasm into Stellaryn's normalized interface model.

## Evidence baseline

Verified on 2026-10-09 against the current Stellar toolchain sources:

- Stellar CLI documents `stellar contract info interface --wasm <WASM> --output json` as a stream of `SCSpecEntry` values.
- Stellar CLI's own implementation uses `soroban_spec::read::from_wasm` when reading a local Wasm contract specification.
- Stellar CLI 28.1.0 pins `soroban-spec 28.0.0` and `stellar-xdr 28.0.0`.
- `soroban-spec 28.0.0` reads the `contractspecv0` Wasm custom section and decodes concatenated `ScSpecEntry` XDR values.
- `stellar-xdr 28.0.0` defines six current `ScSpecEntry` categories: function, struct, union, enum, error enum, and event.

Authoritative references:

- https://developers.stellar.org/docs/tools/cli/stellar-cli#stellar-contract-info-interface
- https://github.com/stellar/stellar-cli/blob/main/cmd/soroban-cli/src/commands/contract/invoke.rs
- https://github.com/stellar/rs-soroban-sdk/blob/v28.0.0/soroban-spec/src/read.rs
- https://github.com/stellar/rs-stellar-xdr/tree/v28.0.0/src/generated

## Extraction strategy

For local Wasm, Stellaryn calls the Rust library directly:

```text
Wasm bytes
  -> soroban_spec::read::from_wasm
  -> Vec<ScSpecEntry>
  -> typed mapping in stellaryn-wasm
  -> ContractInterface
  -> validation + canonical top-level ordering
```

This deliberately avoids invoking an external `stellar` subprocess for local Wasm analysis.

## Current XDR mapping

`ScSpecTypeDef` maps to structured `TypeRef` values.

Scalar/protocol types use stable canonical names such as `Address`, `Symbol`, `Timepoint`, `u64`, and `i128`.

Parameterized types remain structured:

- option;
- result;
- vector;
- map;
- tuple;
- `BytesN { length }`;
- user-defined type references.

Contract entries map as follows:

- `FunctionV0` -> `Function`;
- `UdtStructV0` -> struct `UserType`;
- `UdtUnionV0` -> explicit `UserTypeKind::Union` with zero or positional payload fields;
- `UdtEnumV0` -> `UserTypeKind::Enum` with numeric discriminants;
- `UdtErrorEnumV0` -> `ErrorDefinition`;
- `EventV0` -> `EventDefinition`.

Event parameter location and data format are represented as typed enums, not free-form strings. Phase 5 introduced distinct tagged-union and numeric-enum normalized kinds in interface schema 1.1; the verified XDR categories are preserved rather than collapsed.

## Failure semantics

Malformed Wasm (including invalid trailing bytes), missing, empty, or duplicate `contractspecv0` sections, XDR decoding failures, file I/O failures, and invalid normalized interfaces are errors.

Stellaryn never turns a missing, empty, duplicate, or unreadable contract spec into a successful empty analysis. The official reader normally stops at the first matching custom section; the Phase 10 validation pass inspects the entire module before extraction to reject multiple sections and malformed trailing Wasm.
