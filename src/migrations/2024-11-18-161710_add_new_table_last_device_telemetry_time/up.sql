-- Your SQL goes here
CREATE TABLE IF NOT EXISTS last_device_telemetry_time (
    device_code TEXT NOT NULL PRIMARY KEY,
    record_date TIMESTAMP NOT NULL
);
