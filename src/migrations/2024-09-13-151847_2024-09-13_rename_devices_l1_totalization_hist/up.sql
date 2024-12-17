-- Your SQL goes here
-- This file should undo anything in `up.sql`
ALTER TABLE assets_state_hist RENAME TO devices_l1_totalization_hist;

ALTER TABLE devices_l1_totalization_hist DROP CONSTRAINT assets_state_hist_pkey;

ALTER TABLE devices_l1_totalization_hist ALTER COLUMN asset_reference_id DROP NOT NULL;

ALTER TABLE devices_l1_totalization_hist ADD COLUMN device_code TEXT;

ALTER TABLE devices_l1_totalization_hist ADD COLUMN machine_reference_id INT,
ADD CONSTRAINT devices_l1_totalization_hist_fk_machine_reference_id FOREIGN KEY (machine_reference_id) REFERENCES machines(reference_id);

UPDATE devices_l1_totalization_hist d
SET device_code = a.device_code,
machine_reference_id = a.machine_reference_id
FROM assets a
WHERE a.reference_id = d.asset_reference_id;

ALTER TABLE devices_l1_totalization_hist ALTER COLUMN device_code SET NOT NULL;

ALTER TABLE devices_l1_totalization_hist ADD CONSTRAINT devices_l1_totalization_hist_pkey PRIMARY KEY (device_code, record_date);