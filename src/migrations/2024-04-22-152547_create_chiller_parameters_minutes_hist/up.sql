-- Your SQL goes here
CREATE TABLE IF NOT EXISTS chiller_parameters_minutes_hist (
    unit_id int not null,
    device_code text NOT NULL,
    capa_t DECIMAL(10,2) NOT NULL,
    capb_t DECIMAL(10,2) NOT NULL,
    cap_t DECIMAL(10,2) NOT NULL,
    cpa1_cur DECIMAL(10,2) NOT NULL,
    cpa1_dgt DECIMAL(10,2) NOT NULL,
    cpa1_op DECIMAL(10,2) NOT NULL,
    cpa1_tmp DECIMAL(10,2) NOT NULL,
    cpa2_cur DECIMAL(10,2) NOT NULL,
    cpa2_dgt DECIMAL(10,2) NOT NULL,
    cpa2_op DECIMAL(10,2) NOT NULL,
    cpa2_tmp DECIMAL(10,2) NOT NULL,
    cpb1_cur DECIMAL(10,2) NOT NULL,
    cpb1_dgt DECIMAL(10,2) NOT NULL,
    cpb1_op DECIMAL(10,2) NOT NULL,
    cpb1_tmp DECIMAL(10,2) NOT NULL,
    cpb2_cur DECIMAL(10,2) NOT NULL,
    cpb2_dgt DECIMAL(10,2) NOT NULL,
    cpb2_op DECIMAL(10,2) NOT NULL,
    cpb2_tmp DECIMAL(10,2) NOT NULL,
    cond_ewt DECIMAL(10,2) NOT NULL,
    cond_lwt DECIMAL(10,2) NOT NULL,
    cond_sp DECIMAL(10,2) NOT NULL,
    cool_ewt DECIMAL(10,2) NOT NULL,
    cool_lwt DECIMAL(10,2) NOT NULL,
    ctrl_pnt DECIMAL(10,2) NOT NULL,
    dem_lim DECIMAL(10,2) NOT NULL,
    dp_a DECIMAL(10,2) NOT NULL,
    dp_b DECIMAL(10,2) NOT NULL,
    dop_a1 DECIMAL(10,2) NOT NULL,
    dop_a2 DECIMAL(10,2) NOT NULL,
    dop_b1 DECIMAL(10,2) NOT NULL,
    dop_b2 DECIMAL(10,2) NOT NULL,
    exv_a DECIMAL(10,2) NOT NULL,
    exv_b DECIMAL(10,2) NOT NULL,
    hr_cp_a1 DECIMAL(10,2) NOT NULL,
    hr_cp_a2 DECIMAL(10,2) NOT NULL,
    hr_cp_b1 DECIMAL(10,2) NOT NULL,
    hr_cp_b2 DECIMAL(10,2) NOT NULL,
    lag_lim DECIMAL(10,2) NOT NULL,
    sct_a DECIMAL(10,2) NOT NULL,
    sct_b DECIMAL(10,2) NOT NULL,
    sp DECIMAL(10,2) NOT NULL,
    sp_a DECIMAL(10,2) NOT NULL,
    sp_b DECIMAL(10,2) NOT NULL,
    sst_a DECIMAL(10,2) NOT NULL,
    sst_b DECIMAL(10,2) NOT NULL,
    record_date TIMESTAMP not null,
    PRIMARY KEY(device_code, record_date),
    CONSTRAINT chiller_parameters_minutes_hist_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('chiller_parameters_minutes_hist', 'record_date');
