-- Your SQL goes here
CREATE TABLE IF NOT EXISTS chiller_xa_parameters_minutes_hist (
    unit_id int not null,
    device_code text NOT NULL,
    cap_t DECIMAL(10,2) NOT NULL,
    cond_ewt DECIMAL(10,2) NOT NULL,
    cond_lwt DECIMAL(10,2) NOT NULL,
    cool_ewt DECIMAL(10,2) NOT NULL,
    cool_lwt DECIMAL(10,2) NOT NULL,
    ctrl_pnt DECIMAL(10,2) NOT NULL,
    dp_a DECIMAL(10,2) NOT NULL,
    dp_b DECIMAL(10,2) NOT NULL,
    hr_cp_a DECIMAL(10,2) NOT NULL,
    hr_cp_b DECIMAL(10,2) NOT NULL,
    hr_mach DECIMAL(10,2) NOT NULL,
    hr_mach_b DECIMAL(10,2) NOT NULL,
    oat DECIMAL(10,2) NOT NULL,
    op_a DECIMAL(10,2) NOT NULL,
    op_b DECIMAL(10,2) NOT NULL,
    sct_a DECIMAL(10,2) NOT NULL,
    sct_b DECIMAL(10,2) NOT NULL,
    slt_a DECIMAL(10,2) NOT NULL,
    slt_b DECIMAL(10,2) NOT NULL,
    sp DECIMAL(10,2) NOT NULL,
    sp_a DECIMAL(10,2) NOT NULL,
    sp_b DECIMAL(10,2) NOT NULL,
    sst_a DECIMAL(10,2) NOT NULL,
    sst_b DECIMAL(10,2) NOT NULL,
    record_date TIMESTAMP not null,
    PRIMARY KEY(device_code, record_date),
    CONSTRAINT chiller_xa_parameters_minutes_hist_fk_unit_id FOREIGN KEY (unit_id) REFERENCES units (id)
);

select create_hypertable('chiller_xa_parameters_minutes_hist', 'record_date');

CREATE MATERIALIZED VIEW IF NOT EXISTS chiller_xa_parameters_hist_view
WITH (timescaledb.continuous) AS
SELECT
    cph.unit_id,
    cph.device_code,
    time_bucket('1 hour', cph.record_date) AS compilation_record_date,
    ROUND(AVG(cph.cap_t), 2) AS cap_t,       
    ROUND(AVG(cph.cond_ewt), 2) AS cond_ewt,          
    ROUND(AVG(cph.cond_lwt), 2) AS cond_lwt,          
    ROUND(AVG(cph.cool_ewt), 2) AS cool_ewt,          
    ROUND(AVG(cph.cool_lwt), 2) AS cool_lwt,          
    ROUND(AVG(cph.ctrl_pnt), 2) AS ctrl_pnt,          
    ROUND(AVG(cph.dp_a), 2) AS dp_a,      
    ROUND(AVG(cph.dp_b), 2) AS dp_b,      
    ROUND(AVG(cph.hr_cp_a), 2) AS hr_cp_a,         
    ROUND(AVG(cph.hr_cp_b), 2) AS hr_cp_b,         
    ROUND(AVG(cph.hr_mach), 2) AS hr_mach,         
    ROUND(AVG(cph.hr_mach_b), 2) AS hr_mach_b,           
    ROUND(AVG(cph.oat), 2) AS oat,     
    ROUND(AVG(cph.op_a), 2) AS op_a,      
    ROUND(AVG(cph.op_b), 2) AS op_b,      
    ROUND(AVG(cph.sct_a), 2) AS sct_a,       
    ROUND(AVG(cph.sct_b), 2) AS sct_b,       
    ROUND(AVG(cph.slt_a), 2) AS slt_a,       
    ROUND(AVG(cph.slt_b), 2) AS slt_b,       
    ROUND(AVG(cph.sp), 2) AS sp,
    ROUND(AVG(cph.sp_a), 2) AS sp_a,      
    ROUND(AVG(cph.sp_b), 2) AS sp_b,      
    ROUND(AVG(cph.sst_a), 2) AS sst_a,       
    ROUND(AVG(cph.sst_b), 2) AS sst_b          
FROM
    chiller_xa_parameters_minutes_hist cph
GROUP BY
    compilation_record_date,
    cph.unit_id,
    cph.device_code
WITH NO DATA;

SELECT add_continuous_aggregate_policy('chiller_xa_parameters_hist_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour');
