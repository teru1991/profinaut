# IR US Official Filing Family Mapping v1

Source filing form-like metadata must be retained while mapping to canonical family.

Minimum mapping classes:
- annual (`10-K`) -> `statutory_annual`
- quarterly (`10-Q`) -> `statutory_quarterly`
- current (`8-K`) -> `statutory_current`
- proxy (`DEF 14A`) -> `proxy`
- registration-like (`S-*`) -> `misc_ir_document`
- insider-like (`3/4/5`) -> `misc_ir_document`
- other -> `misc_ir_document`
