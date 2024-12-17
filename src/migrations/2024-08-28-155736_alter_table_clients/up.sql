-- Your SQL goes here
ALTER TABLE CLIENTS ADD COLUMN amount_minutes_check_offline INTEGER DEFAULT NULL;

UPDATE CLIENTS SET amount_minutes_check_offline = 30 WHERE client_name = 'BANCO DO BRASIL';
