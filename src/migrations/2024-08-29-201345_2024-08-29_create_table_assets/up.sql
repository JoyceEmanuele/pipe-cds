-- Your SQL goes here
ALTER TABLE machines ADD CONSTRAINT machines_uk_reference_id UNIQUE (reference_id);

CREATE TABLE IF NOT EXISTS assets (
    id SERIAL PRIMARY KEY,
    unit_id int not null,
    asset_name text not null,
    device_code text not null,
    machine_reference_id int not null,
    reference_id int not null,

    CONSTRAINT assets_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id),  
    CONSTRAINT assets_uk_referecen_id UNIQUE (reference_id),      
    CONSTRAINT assets_fk_machine_reference_id FOREIGN KEY (machine_reference_id) REFERENCES machines (reference_id)
);
