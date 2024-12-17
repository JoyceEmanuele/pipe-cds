-- Your SQL goes here
ALTER TABLE UNITS ADD COLUMN tarifa_kwh decimal(8, 2) DEFAULT NULL;
ALTER TABLE UNITS ADD COLUMN constructed_area decimal(8, 2) DEFAULT NULL;
ALTER TABLE UNITS ADD COLUMN capacity_power decimal(8, 2) DEFAULT NULL;