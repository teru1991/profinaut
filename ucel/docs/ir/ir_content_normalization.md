# IR Content Normalization
Supported normalized formats: html, pdf, xbrl, ixbrl, xml, txt, csv, json, rss, zip.
Pipeline: format detection -> format normalizer -> normalized content assembly.
Includes sections/tables/attachments/provenance and schema version.
binary-free fixture policy: repository stores text fixtures only; binary bytes (pdf/zip) are built at runtime during tests.
