-- Your SQL goes here
CREATE TABLE IF NOT EXISTS assets_state_hist (
    asset_reference_id int not null,
    record_date DATE not null,
    seconds_on int not null,
    seconds_off int not null,
    seconds_on_outside_programming int not null,
    seconds_must_be_off int not null,
    percentage_on_outside_programming decimal(5,2) not null,
    programming text not null,
    PRIMARY KEY(asset_reference_id, record_date),

    CONSTRAINT assets_state_hist_fk_asset_reference_id FOREIGN KEY (asset_reference_id) REFERENCES assets (reference_id)
);

select create_hypertable('assets_state_hist', 'record_date');