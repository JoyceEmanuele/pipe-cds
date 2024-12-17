-- Your SQL goes here
CREATE TABLE IF NOT EXISTS water_consumption_forecast (
    unit_id int not null,
    forecast_date DATE not null,
    monday DECIMAL(10,2),
    tuesday DECIMAL(10,2),
    wednesday DECIMAL(10,2),
    thursday DECIMAL(10,2),
    friday DECIMAL(10,2),
    saturday DECIMAL(10,2),
    sunday DECIMAL(10,2),
    PRIMARY KEY(unit_id, forecast_date),

    CONSTRAINT water_consumption_forecast_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('water_consumption_forecast', 'forecast_date');
