-- Your SQL goes here

CREATE TABLE IF NOT EXISTS energy_consumption_forecast (
    electric_circuit_id int not null,
    consumption_forecast DECIMAL(10,2) not null,
    record_date TIMESTAMP not null,
    PRIMARY KEY(electric_circuit_id, record_date),

    CONSTRAINT energy_consumption_forecast_fk_electric_circuit_id FOREIGN KEY (electric_circuit_id) REFERENCES electric_circuits (id)
);

select create_hypertable('energy_consumption_forecast', 'record_date');