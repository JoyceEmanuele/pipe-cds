-- This file should undo anything in `up.sql`
drop materialized view IF EXISTS chiller_xa_hvar_parameters_hist_view;

ALTER TABLE chiller_xa_hvar_parameters_minutes_hist
ADD COLUMN cond_ewt DECIMAL(10,2) NOT NULL,
ADD COLUMN cond_lwt DECIMAL(10,2) NOT NULL;

CREATE MATERIALIZED VIEW IF NOT EXISTS chiller_xa_hvar_parameters_hist_view
WITH (timescaledb.continuous) AS
SELECT
    ch.unit_id,
    ch.device_code,
    time_bucket('1 hour', ch.record_date) AS compilation_record_date,
    ROUND(AVG(ch.genunit_ui), 2) AS genunit_ui,
    ROUND(AVG(ch.cap_t), 2) AS cap_t,
    ROUND(AVG(ch.tot_curr), 2) AS tot_curr,
    ROUND(AVG(ch.ctrl_pnt), 2) AS ctrl_pnt,
    ROUND(AVG(ch.oat), 2) AS oat,
    ROUND(AVG(ch.cool_ewt), 2) AS cool_ewt,
    ROUND(AVG(ch.cool_lwt), 2) AS cool_lwt,
    ROUND(AVG(ch.cond_ewt), 2) AS cond_ewt,
    ROUND(AVG(ch.cond_lwt), 2) AS cond_lwt,
    ROUND(AVG(ch.circa_an_ui), 2) AS circa_an_ui,
    ROUND(AVG(ch.capa_t), 2) AS capa_t,
    ROUND(AVG(ch.dp_a), 2) AS dp_a,
    ROUND(AVG(ch.sp_a), 2) AS sp_a,
    ROUND(AVG(ch.econ_p_a), 2) AS econ_p_a,
    ROUND(AVG(ch.op_a), 2) AS op_a,
    ROUND(AVG(ch.dop_a), 2) AS dop_a,
    ROUND(AVG(ch.curren_a), 2) AS curren_a,
    ROUND(AVG(ch.cp_tmp_a), 2) AS cp_tmp_a,
    ROUND(AVG(ch.dgt_a), 2) AS dgt_a,
    ROUND(AVG(ch.eco_tp_a), 2) AS eco_tp_a,
    ROUND(AVG(ch.sct_a), 2) AS sct_a,
    ROUND(AVG(ch.sst_a), 2) AS sst_a,
    ROUND(AVG(ch.sst_b), 2) AS sst_b,
    ROUND(AVG(ch.circb_an_ui), 2) AS circb_an_ui,
    ROUND(AVG(ch.circc_an_ui), 2) AS circc_an_ui,
    ROUND(AVG(ch.capb_t), 2) AS capb_t,
    ROUND(AVG(ch.dp_b), 2) AS dp_b,
    ROUND(AVG(ch.sp_b), 2) AS sp_b,
    ROUND(AVG(ch.econ_p_b), 2) AS econ_p_b,
    ROUND(AVG(ch.suct_t_a), 2) AS suct_t_a,
    ROUND(AVG(ch.exv_a), 2) AS exv_a,
    ROUND(AVG(ch.op_b), 2) AS op_b,
    ROUND(AVG(ch.dop_b), 2) AS dop_b,
    ROUND(AVG(ch.curren_b), 2) AS curren_b,
    ROUND(AVG(ch.cp_tmp_b), 2) AS cp_tmp_b,
    ROUND(AVG(ch.dgt_b), 2) AS dgt_b,
    ROUND(AVG(ch.eco_tp_b), 2) AS eco_tp_b,
    ROUND(AVG(ch.sct_b), 2) AS sct_b,
    ROUND(AVG(ch.suct_t_b), 2) AS suct_t_b,
    ROUND(AVG(ch.exv_b), 2) AS exv_b,
    ROUND(AVG(ch.capc_t), 2) AS capc_t,
    ROUND(AVG(ch.dp_c), 2) AS dp_c,
    ROUND(AVG(ch.sp_c), 2) AS sp_c,
    ROUND(AVG(ch.econ_p_c), 2) AS econ_p_c,
    ROUND(AVG(ch.op_c), 2) AS op_c,
    ROUND(AVG(ch.dop_c), 2) AS dop_c,
    ROUND(AVG(ch.curren_c), 2) AS curren_c,
    ROUND(AVG(ch.cp_tmp_c), 2) AS cp_tmp_c,
    ROUND(AVG(ch.dgt_c), 2) AS dgt_c,
    ROUND(AVG(ch.eco_tp_c), 2) AS eco_tp_c,
    ROUND(AVG(ch.sct_c), 2) AS sct_c,
    ROUND(AVG(ch.sst_c), 2) AS sst_c,
    ROUND(AVG(ch.suct_t_c), 2) AS suct_t_c,
    ROUND(AVG(ch.exv_c), 2) AS exv_c
FROM
    chiller_xa_hvar_parameters_minutes_hist ch
GROUP BY
    compilation_record_date,
    ch.unit_id,
    ch.device_code
WITH NO DATA;

SELECT add_continuous_aggregate_policy('chiller_xa_hvar_parameters_hist_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour');
