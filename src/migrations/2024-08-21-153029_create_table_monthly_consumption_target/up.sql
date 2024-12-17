-- Your SQL goes here

CREATE TABLE IF NOT EXISTS energy_monthly_consumption_target (
    unit_id int not null,
    consumption_target DECIMAL(10,2) not null,
    date_forecast TIMESTAMP not null,
    PRIMARY KEY(unit_id, date_forecast),

    CONSTRAINT energy_monthly_consumption_target_fk_units_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('energy_monthly_consumption_target', 'date_forecast');