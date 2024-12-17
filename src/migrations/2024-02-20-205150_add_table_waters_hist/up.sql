-- Your SQL goes here
CREATE TABLE IF NOT EXISTS waters_hist (
    unit_id int not null,
    supplier text NOT NULL,
    device_code text NOT NULL,
    consumption DECIMAL(10,2) not null,
    record_date DATE not null,

    PRIMARY KEY(unit_id, record_date),

    CONSTRAINT waters_hist_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('waters_hist', 'record_date');

