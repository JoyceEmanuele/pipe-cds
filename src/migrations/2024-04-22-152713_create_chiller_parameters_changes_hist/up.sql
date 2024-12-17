-- Your SQL goes here
CREATE TABLE IF NOT EXISTS chiller_parameters_changes_hist (
    unit_id int not null,
    device_code text NOT NULL,
    parameter_name text not null,
    parameter_value int not null,
    record_date TIMESTAMP not null,
    PRIMARY KEY(device_code, parameter_name, record_date),
    CONSTRAINT chiller_parameters_changes_hist_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('chiller_parameters_changes_hist', 'record_date');
