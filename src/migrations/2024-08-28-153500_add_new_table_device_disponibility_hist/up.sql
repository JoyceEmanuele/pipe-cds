-- Your SQL goes here
CREATE TABLE IF NOT EXISTS device_disponibility_hist (
    unit_id int not null,
    device_code text NOT NULL,
    disponibility DECIMAL(5,2) not null,
    record_date DATE not null,
    PRIMARY KEY(unit_id, device_code, record_date),

    CONSTRAINT device_disponibility_hist_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('device_disponibility_hist', 'record_date');
