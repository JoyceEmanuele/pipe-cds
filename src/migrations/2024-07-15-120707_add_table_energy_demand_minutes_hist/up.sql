-- Your SQL goes here
CREATE TABLE IF NOT EXISTS energy_demand_minutes_hist (
    average_demand DECIMAL(10,2) not null,
    electric_circuit_id int not null,
    max_demand DECIMAL (10,2) not null,
    min_demand DECIMAL (10,2) not null,
    record_date TIMESTAMP not null,
    PRIMARY KEY(electric_circuit_id, record_date),

    CONSTRAINT demand_energy_minutes_hist_fk_electric_circuit_id FOREIGN KEY (electric_circuit_id) REFERENCES electric_circuits (id)
);

select create_hypertable('energy_demand_minutes_hist', 'record_date');
