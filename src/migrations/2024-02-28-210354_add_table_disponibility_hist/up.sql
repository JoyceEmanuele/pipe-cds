-- Your SQL goes here
CREATE TABLE IF NOT EXISTS disponibility_hist (
    unit_id int not null,
    disponibility DECIMAL(5,2) not null,
    record_date DATE not null,
    PRIMARY KEY(unit_id, record_date),

    CONSTRAINT disponibility_hist_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('disponibility_hist', 'record_date');
