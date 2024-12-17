-- Your SQL goes here
CREATE TABLE IF NOT EXISTS electric_circuits (
    id SERIAL PRIMARY KEY,
    unit_id int not null,
    name VARCHAR(50) NOT NULL,
    reference_id int not null,

    CONSTRAINT electric_circuits_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);
