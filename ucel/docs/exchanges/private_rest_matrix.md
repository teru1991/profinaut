# Private REST Matrix (JP-resident policy baseline)

This matrix tracks canonical UCEL private REST support states for current registry venues.

| Venue | Policy scope | balances | open_orders | get_order | cancel | fills | account_profile | positions | Notes |
|---|---|---|---|---|---|---|---|---|---|
| bitbank | public_private | supported | supported/partial | supported/partial | supported | supported/partial | partial | partial | canonical private ops are enabled by policy |
| bitflyer | public_private | supported | supported/partial | supported/partial | supported | supported/partial | partial | partial | child/parent-order differences normalized in upper layers |
| coincheck | public_private | supported | supported/partial | supported/partial | supported | supported/partial | partial | partial | numeric-string parsing handled in canonical model layer |
| gmocoin | public_private | supported | supported/partial | supported/partial | supported | supported/partial | partial | partial | code/message reject mapping required |
| bittrade | public_only (default) | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | private REST remains blocked by current JP policy SSOT |
| sbivc | public_only | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked before network send |
| upbit | public_only (default) | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | blocked_by_policy | no private allow evidence in current policy |
