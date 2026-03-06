# WS Ingest Matrix

| venue family | public ingest | private ingest | ack | heartbeat | integrity | restart resume |
|---|---|---|---|---|---|---|
| binance spot | yes | yes | implicit/explicit | yes | seq+checksum | yes |
| bybit linear | yes | yes | explicit | yes | seq+checksum | yes |
| okx swap | yes | yes | explicit | yes | checksum | yes |
| kraken spot | yes | partial | explicit | yes | checksum | yes |
| bithumb spot | yes | n/a | implicit | yes | sequence | yes |
