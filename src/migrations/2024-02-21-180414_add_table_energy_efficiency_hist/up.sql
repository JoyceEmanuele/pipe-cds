-- Your SQL goes here
CREATE TABLE IF NOT EXISTS energy_efficiency_hist (
    machine_id int not null,
    device_code text not null,
    capacity_power DECIMAL(10,2) not null,
    consumption DECIMAL(10,2) not null,
    utilization_time DECIMAL(10,2),
    record_date DATE not null,
    PRIMARY KEY(device_code, record_date),

    CONSTRAINT energy_efficiency_hist_fk_machine_id FOREIGN KEY (machine_id) REFERENCES machines (id)
);

select create_hypertable('energy_efficiency_hist', 'record_date');
