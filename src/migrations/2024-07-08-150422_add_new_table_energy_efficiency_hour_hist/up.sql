-- Your SQL goes here
CREATE TABLE IF NOT EXISTS energy_efficiency_hour_hist (
    machine_id int not null,
    device_code text not null,
    consumption DECIMAL(10,3) not null,
    utilization_time DECIMAL(10,3),
    record_date TIMESTAMP not null,
    PRIMARY KEY(device_code, record_date),

    CONSTRAINT energy_efficiency_hour_hist_fk_machine_id FOREIGN KEY (machine_id) REFERENCES machines (id)
);

select create_hypertable('energy_efficiency_hour_hist', 'record_date');
