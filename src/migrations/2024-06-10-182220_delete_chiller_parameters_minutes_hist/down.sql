-- This file should undo anything in `up.sql`
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

CREATE MATERIALIZED VIEW IF NOT EXISTS chiller_parameters_hist_view
WITH (timescaledb.continuous) AS
SELECT
    cph.unit_id,
    cph.device_code,
    time_bucket('1 hour', cph.record_date) AS compilation_record_date,
    ROUND(AVG(cph.cap_t), 2) AS cap_t,
    ROUND(AVG(cph.dem_lim), 2) AS dem_lim,
    ROUND(AVG(cph.lag_lim), 2) AS lag_lim,
    ROUND(AVG(cph.sp), 2) AS sp,
    ROUND(AVG(cph.ctrl_pnt), 2) AS ctrl_pnt,
    ROUND(AVG(cph.capa_t), 2) AS capa_t,
    ROUND(AVG(cph.dp_a), 2) AS dp_a,
    ROUND(AVG(cph.sp_a), 2) AS sp_a,
    ROUND(AVG(cph.sct_a), 2) AS sct_a,
    ROUND(AVG(cph.sst_a), 2) AS sst_a,
    ROUND(AVG(cph.capb_t), 2) AS capb_t,
    ROUND(AVG(cph.dp_b), 2) AS dp_b,
    ROUND(AVG(cph.sp_b), 2) AS sp_b,
    ROUND(AVG(cph.sct_b), 2) AS sct_b,
    ROUND(AVG(cph.sst_b), 2) AS sst_b,
    ROUND(AVG(cph.cond_lwt), 2) AS cond_lwt,
    ROUND(AVG(cph.cond_ewt), 2) AS cond_ewt,
    ROUND(AVG(cph.cool_lwt), 2) AS cool_lwt,
    ROUND(AVG(cph.cool_ewt), 2) AS cool_ewt,
    ROUND(AVG(cph.cpa1_op), 2) AS cpa1_op,
    ROUND(AVG(cph.cpa2_op), 2) AS cpa2_op,
    ROUND(AVG(cph.dop_a1), 2) AS dop_a1,
    ROUND(AVG(cph.dop_a2), 2) AS dop_a2,
    ROUND(AVG(cph.cpa1_dgt), 2) AS cpa1_dgt,
    ROUND(AVG(cph.cpa2_dgt), 2) AS cpa2_dgt,
    ROUND(AVG(cph.exv_a), 2) AS exv_a,
    ROUND(AVG(cph.hr_cp_a1), 2) AS hr_cp_a1,
    ROUND(AVG(cph.hr_cp_a2), 2) AS hr_cp_a2,
    ROUND(AVG(cph.cpa1_tmp), 2) AS cpa1_tmp,
    ROUND(AVG(cph.cpa2_tmp), 2) AS cpa2_tmp,
    ROUND(AVG(cph.cpa1_cur), 2) AS cpa1_cur,
    ROUND(AVG(cph.cpa2_cur), 2) AS cpa2_cur,
    ROUND(AVG(cph.cpb1_op), 2) AS cpb1_op,
    ROUND(AVG(cph.cpb2_op), 2) AS cpb2_op,
    ROUND(AVG(cph.dop_b1), 2) AS dop_b1,
    ROUND(AVG(cph.dop_b2), 2) AS dop_b2,
    ROUND(AVG(cph.cpb1_dgt), 2) AS cpb1_dgt,
    ROUND(AVG(cph.cpb2_dgt), 2) AS cpb2_dgt,
    ROUND(AVG(cph.exv_b), 2) AS exv_b,
    ROUND(AVG(cph.hr_cp_b1), 2) AS hr_cp_b1,
    ROUND(AVG(cph.hr_cp_b2), 2) AS hr_cp_b2,
    ROUND(AVG(cph.cpb1_tmp), 2) AS cpb1_tmp,
    ROUND(AVG(cph.cpb2_tmp), 2) AS cpb2_tmp,
    ROUND(AVG(cph.cpb1_cur), 2) AS cpb1_cur,
    ROUND(AVG(cph.cpb2_cur), 2) AS cpb2_cur,
    ROUND(AVG(cph.cond_sp), 2) AS cond_sp
FROM
    chiller_parameters_minutes_hist cph
GROUP BY
    compilation_record_date,
    cph.unit_id,
    cph.device_code
WITH NO DATA;
SELECT add_continuous_aggregate_policy('chiller_parameters_hist_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour');