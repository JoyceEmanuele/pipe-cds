-- This file should undo anything in `up.sql`
ALTER TABLE devices_l1_totalization_hist DROP CONSTRAINT devices_l1_totalization_hist_pkey;

ALTER TABLE devices_l1_totalization_hist DROP CONSTRAINT devices_l1_totalization_hist_fk_machine_reference_id;

ALTER TABLE devices_l1_totalization_hist DROP COLUMN machine_reference_id;

ALTER TABLE devices_l1_totalization_hist DROP COLUMN device_code;

ALTER TABLE devices_l1_totalization_hist ALTER COLUMN asset_reference_id SET NOT NULL;

ALTER TABLE devices_l1_totalization_hist ADD CONSTRAINT assets_state_hist_pkey PRIMARY KEY (asset_reference_id, record_date);

ALTER TABLE devices_l1_totalization_hist RENAME TO assets_state_hist;