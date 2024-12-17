-- Your SQL goes here
CREATE TABLE IF NOT EXISTS water_hist (
    unit_id int not null,
    supplier text NOT NULL,
    device_code text NOT NULL,
    consumption DECIMAL(10,2) not null,
    record_date TIMESTAMP not null,
    is_measured_consumption BOOLEAN DEFAULT false,
    is_valid_consumption BOOLEAN DEFAULT true,

    PRIMARY KEY(unit_id, record_date),

    CONSTRAINT water_hist_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('water_hist', 'record_date');
