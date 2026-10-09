# Custom Type Compatibility (Phase 5)

Stellaryn compares the normalized `UserType` collections without touching the Wasm parser. Numeric XDR enums and tagged XDR unions are now **different `UserTypeKind` variants**, so changing between them cannot silently appear compatible.

## Rule policy

| Rule | Classification |
| --- | --- |
| `TYPE_ADDED` | NON_BREAKING |
| `TYPE_REMOVED` | BREAKING |
| `TYPE_KIND_CHANGED` | BREAKING |
| `STRUCT_FIELD_ADDED` | BREAKING |
| `STRUCT_FIELD_REMOVED` | BREAKING |
| `STRUCT_FIELD_TYPE_CHANGED` | BREAKING |
| `STRUCT_FIELD_REORDERED` | REVIEW_REQUIRED |
| `ENUM_VARIANT_ADDED` | REVIEW_REQUIRED |
| `ENUM_VARIANT_REMOVED` | BREAKING |
| `ENUM_DISCRIMINANT_CHANGED` | BREAKING |
| `UNION_VARIANT_ADDED` | REVIEW_REQUIRED |
| `UNION_VARIANT_REMOVED` | BREAKING |
| `UNION_PAYLOAD_COUNT_CHANGED` | BREAKING |
| `UNION_PAYLOAD_TYPE_CHANGED` | BREAKING |
| `UNION_PAYLOAD_NAME_CHANGED` | REVIEW_REQUIRED |

Review-needed findings are deliberately not described as proven compatibility. Enum/union variant additions may affect exhaustive consumers; struct field ordering can matter to generated clients, although some Soroban encodings use named keys. Changing the field set or field type is conservatively breaking.

Variant matching uses stable names, not declaration positions. Numeric-enum discriminants are compared explicitly, and union payload entries are matched positionally. Pure documentation changes and enum variant declaration reorderings do not produce findings. All rule output is deterministically sorted by classification, rule ID, subject, and summary.

## Data-model revision

`INTERFACE_SCHEMA_VERSION` moved from `1.0` to `1.1` when `UserTypeKind::Union` was introduced. Numeric `Enum` variants require numeric discriminants and cannot carry payloads; `Union` variants cannot have numeric discriminants. Duplicate numeric discriminants are rejected.

## Boundaries and limitations

The custom-type engine itself does not compare public errors or events, aggregate the whole-contract verdict, or render reports; those are handled by the **already implemented** later engines and CLI. It does not prove runtime behavior or upgrade safety. Findings identify **spec-level compatibility changes** only.
