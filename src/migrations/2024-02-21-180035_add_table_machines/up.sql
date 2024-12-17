-- Your SQL goes here
CREATE TABLE IF NOT EXISTS machines (
    id SERIAL PRIMARY KEY,
    unit_id int not null,
    machine_name text not null,
    reference_id int not null,

    CONSTRAINT machines_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);
