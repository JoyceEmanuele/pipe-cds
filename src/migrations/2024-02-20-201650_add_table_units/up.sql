-- Your SQL goes here
CREATE TABLE IF NOT EXISTS units (
    id SERIAL PRIMARY KEY,
    client_id int not null,
    unit_name varchar(250) not null,
    reference_id int not null,

    CONSTRAINT units_fk_client_id FOREIGN KEY (client_id) REFERENCES clients (id)
);
