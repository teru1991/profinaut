# Domestic Public WS Extension Runtime Matrix

| venue | operation_id | readiness_mode | integrity_mode | resume_mode | snapshot_prerequisite |
| --- | --- | --- | --- | --- | --- |
| bitbank | `crypto.public.ws.market.circuit-break-info` | explicit_ack | none | resubscribe_only | no |
| bitbank | `crypto.public.ws.market.transactions` | implicit_observation | sequence_only | resubscribe_only | no |
| bitflyer | `crypto.public.ws.board` | implicit_observation | sequence_and_checksum | resnapshot_then_resubscribe | yes |
| bitflyer | `crypto.public.ws.board_snapshot` | implicit_observation | snapshot_only | resnapshot_then_resubscribe | yes |
| bitflyer | `crypto.public.ws.executions` | implicit_observation | sequence_only | resubscribe_only | no |
| bitflyer | `fx.public.ws.board` | implicit_observation | sequence_and_checksum | resnapshot_then_resubscribe | yes |
| bitflyer | `fx.public.ws.board_snapshot` | implicit_observation | snapshot_only | resnapshot_then_resubscribe | yes |
| bitflyer | `fx.public.ws.executions` | implicit_observation | sequence_only | resubscribe_only | no |
| bittrade | `public.ws.market.bbo` | immediate_active | none | resubscribe_only | no |
| bittrade | `public.ws.market.detail` | immediate_active | none | resubscribe_only | no |
