# Normalized Contract Interface

Phase 2 defines Stellaryn's comparison-first contract interface model.

The model is deliberately independent from any particular Stellar CLI JSON layout or XDR Rust type. Phase 3 will map verified Soroban contract specification data into this model.

## Design goals

- deterministic serialization;
- explicit structured types rather than ad-hoc type strings;
- stable top-level identifiers;
- preservation of semantically meaningful member order;
- typed validation failures for malformed or ambiguous normalized data;
- enough structure to support later function, custom-type, error, and event compatibility rules.

## Type references

`TypeRef` supports:

- primitive names;
- named contract types;
- option;
- result;
- vector;
- map;
- tuple.

Protocol-specific primitive names are not hard-coded in Phase 2. The Phase 3 extractor is responsible for mapping verified Soroban spec types to canonical primitive/named tokens.

## Ordering

Normalization sorts only top-level collections that do not carry semantic order:

- functions by name;
- user-defined types by name;
- error definitions by name;
- events by name;
- error cases by numeric value then name.

It preserves:

- function parameter order;
- function output order;
- struct field order;
- enum variant order;
- variant payload order;
- event prefix-topic order;
- event parameter order.

This distinction is important because later compatibility rules may need to detect reordering.

## Stable IDs

Top-level identifiers are derived deterministically:

```text
function:<name>
type:<name>
error:<name>
event:<name>
```

Nested compatibility records will use parent identifiers plus positional/name evidence in later phases.

## Validation

The model rejects:

- unsupported interface schema versions;
- empty names;
- duplicate top-level names within a category;
- duplicate function/event parameters;
- duplicate struct fields;
- duplicate enum variants;
- duplicate named variant fields;
- duplicate error case names;
- duplicate numeric values inside one error definition;
- empty primitive or named-type tokens.

Validation does not attempt to prove Soroban protocol correctness. That responsibility belongs to the verified extraction layer in Phase 3.
