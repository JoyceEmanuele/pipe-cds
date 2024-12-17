-- Your SQL goes here
CREATE TABLE IF NOT EXISTS clients (
    id SERIAL PRIMARY KEY,
    client_name varchar(250) not null
);
