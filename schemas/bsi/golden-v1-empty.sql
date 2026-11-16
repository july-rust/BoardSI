-- F_004 independent, immutable synthetic v1 semantic seed.
-- Apply only after schemas/bsi/v1.sql to a NEW SQLite database.
-- No private board data. Stable file UUID and producer metadata.
INSERT INTO file_meta
(singleton,file_uuid,format_major,format_minor,min_reader_major,min_reader_minor,board_kind,producer,producer_version,created_utc)
VALUES
(1,'00000000-0000-4000-8000-000000000004',1,0,1,0,'Pcb','F004-independent-oracle','1','2026-11-16T00:00:00Z');
