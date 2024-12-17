-- This file should undo anything in `up.sql`
ALTER TABLE UNITS DROP COLUMN tarifa_kwh;
ALTER TABLE UNITS DROP COLUMN constructed_area;
ALTER TABLE UNITS DROP COLUMN capacity_power;