// @generated automatically by Diesel CLI.

diesel::table! {
    assets (id) {
        id -> Int4,
        unit_id -> Int4,
        asset_name -> Text,
        device_code -> Text,
        machine_reference_id -> Int4,
        reference_id -> Int4,
    }
}

diesel::table! {
    chiller_hx_parameters_minutes_hist (device_code, record_date) {
        unit_id -> Int4,
        device_code -> Text,
        capa_t -> Numeric,
        capb_t -> Numeric,
        cap_t -> Numeric,
        cpa1_cur -> Numeric,
        cpa1_dgt -> Numeric,
        cpa1_op -> Numeric,
        cpa1_tmp -> Numeric,
        cpa2_cur -> Numeric,
        cpa2_dgt -> Numeric,
        cpa2_op -> Numeric,
        cpa2_tmp -> Numeric,
        cpb1_cur -> Numeric,
        cpb1_dgt -> Numeric,
        cpb1_op -> Numeric,
        cpb1_tmp -> Numeric,
        cpb2_cur -> Numeric,
        cpb2_dgt -> Numeric,
        cpb2_op -> Numeric,
        cpb2_tmp -> Numeric,
        cond_ewt -> Numeric,
        cond_lwt -> Numeric,
        cond_sp -> Numeric,
        cool_ewt -> Numeric,
        cool_lwt -> Numeric,
        ctrl_pnt -> Numeric,
        dem_lim -> Numeric,
        dp_a -> Numeric,
        dp_b -> Numeric,
        dop_a1 -> Numeric,
        dop_a2 -> Numeric,
        dop_b1 -> Numeric,
        dop_b2 -> Numeric,
        exv_a -> Numeric,
        exv_b -> Numeric,
        hr_cp_a1 -> Numeric,
        hr_cp_a2 -> Numeric,
        hr_cp_b1 -> Numeric,
        hr_cp_b2 -> Numeric,
        lag_lim -> Numeric,
        sct_a -> Numeric,
        sct_b -> Numeric,
        sp -> Numeric,
        sp_a -> Numeric,
        sp_b -> Numeric,
        sst_a -> Numeric,
        sst_b -> Numeric,
        record_date -> Timestamp,
    }
}

diesel::table! {
    chiller_parameters_changes_hist (device_code, parameter_name, record_date) {
        unit_id -> Int4,
        device_code -> Text,
        parameter_name -> Text,
        parameter_value -> Int4,
        record_date -> Timestamp,
    }
}

diesel::table! {
    chiller_xa_hvar_parameters_minutes_hist (device_code, record_date) {
        unit_id -> Int4,
        device_code -> Text,
        genunit_ui -> Numeric,
        cap_t -> Numeric,
        tot_curr -> Numeric,
        ctrl_pnt -> Numeric,
        oat -> Numeric,
        cool_ewt -> Numeric,
        cool_lwt -> Numeric,
        circa_an_ui -> Numeric,
        capa_t -> Numeric,
        dp_a -> Numeric,
        sp_a -> Numeric,
        econ_p_a -> Numeric,
        op_a -> Numeric,
        dop_a -> Numeric,
        curren_a -> Numeric,
        cp_tmp_a -> Numeric,
        dgt_a -> Numeric,
        eco_tp_a -> Numeric,
        sct_a -> Numeric,
        sst_a -> Numeric,
        sst_b -> Numeric,
        suct_t_a -> Numeric,
        exv_a -> Numeric,
        circb_an_ui -> Numeric,
        capb_t -> Numeric,
        dp_b -> Numeric,
        sp_b -> Numeric,
        econ_p_b -> Numeric,
        op_b -> Numeric,
        dop_b -> Numeric,
        curren_b -> Numeric,
        cp_tmp_b -> Numeric,
        dgt_b -> Numeric,
        eco_tp_b -> Numeric,
        sct_b -> Numeric,
        suct_t_b -> Numeric,
        exv_b -> Numeric,
        circc_an_ui -> Numeric,
        capc_t -> Numeric,
        dp_c -> Numeric,
        sp_c -> Numeric,
        econ_p_c -> Numeric,
        op_c -> Numeric,
        dop_c -> Numeric,
        curren_c -> Numeric,
        cp_tmp_c -> Numeric,
        dgt_c -> Numeric,
        eco_tp_c -> Numeric,
        sct_c -> Numeric,
        sst_c -> Numeric,
        suct_t_c -> Numeric,
        exv_c -> Numeric,
        record_date -> Timestamp,
    }
}

diesel::table! {
    chiller_xa_parameters_minutes_hist (device_code, record_date) {
        unit_id -> Int4,
        device_code -> Text,
        cap_t -> Numeric,
        cond_ewt -> Numeric,
        cond_lwt -> Numeric,
        cool_ewt -> Numeric,
        cool_lwt -> Numeric,
        ctrl_pnt -> Numeric,
        dp_a -> Numeric,
        dp_b -> Numeric,
        hr_cp_a -> Numeric,
        hr_cp_b -> Numeric,
        hr_mach -> Numeric,
        hr_mach_b -> Numeric,
        oat -> Numeric,
        op_a -> Numeric,
        op_b -> Numeric,
        sct_a -> Numeric,
        sct_b -> Numeric,
        slt_a -> Numeric,
        slt_b -> Numeric,
        sp -> Numeric,
        sp_a -> Numeric,
        sp_b -> Numeric,
        sst_a -> Numeric,
        sst_b -> Numeric,
        record_date -> Timestamp,
    }
}

diesel::table! {
    clients (id) {
        id -> Int4,
        #[max_length = 250]
        client_name -> Varchar,
        amount_minutes_check_offline -> Nullable<Int4>,
    }
}

diesel::table! {
    device_disponibility_hist (unit_id, device_code, record_date) {
        unit_id -> Int4,
        device_code -> Text,
        disponibility -> Numeric,
        record_date -> Date,
    }
}

diesel::table! {
    devices_l1_totalization_hist (device_code, record_date) {
        asset_reference_id -> Nullable<Int4>,
        record_date -> Date,
        seconds_on -> Int4,
        seconds_off -> Int4,
        seconds_on_outside_programming -> Int4,
        seconds_must_be_off -> Int4,
        percentage_on_outside_programming -> Numeric,
        programming -> Text,
        device_code -> Text,
        machine_reference_id -> Nullable<Int4>,
    }
}

diesel::table! {
    disponibility_hist (unit_id, record_date) {
        unit_id -> Int4,
        disponibility -> Numeric,
        record_date -> Date,
    }
}

diesel::table! {
    electric_circuits (id) {
        id -> Int4,
        unit_id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        reference_id -> Int4,
    }
}

diesel::table! {
    energy_consumption_forecast (electric_circuit_id, record_date) {
        electric_circuit_id -> Int4,
        consumption_forecast -> Numeric,
        record_date -> Timestamp,
    }
}

diesel::table! {
    energy_demand_minutes_hist (electric_circuit_id, record_date) {
        average_demand -> Numeric,
        electric_circuit_id -> Int4,
        max_demand -> Numeric,
        min_demand -> Numeric,
        record_date -> Timestamp,
    }
}

diesel::table! {
    energy_efficiency_hist (device_code, record_date) {
        machine_id -> Int4,
        device_code -> Text,
        capacity_power -> Numeric,
        consumption -> Numeric,
        utilization_time -> Nullable<Numeric>,
        record_date -> Date,
    }
}

diesel::table! {
    energy_efficiency_hour_hist (device_code, record_date) {
        machine_id -> Int4,
        device_code -> Text,
        consumption -> Numeric,
        utilization_time -> Nullable<Numeric>,
        record_date -> Timestamp,
    }
}

diesel::table! {
    energy_hist (electric_circuit_id, record_date) {
        electric_circuit_id -> Int4,
        consumption -> Numeric,
        record_date -> Timestamp,
        is_measured_consumption -> Nullable<Bool>,
        is_valid_consumption -> Nullable<Bool>,
    }
}

diesel::table! {
    energy_monthly_consumption_target (unit_id, date_forecast) {
        unit_id -> Int4,
        consumption_target -> Numeric,
        date_forecast -> Timestamp,
    }
}

diesel::table! {
    last_device_telemetry_time (device_code) {
        device_code -> Text,
        record_date -> Timestamp,
    }
}

diesel::table! {
    machines (id) {
        id -> Int4,
        unit_id -> Int4,
        machine_name -> Text,
        reference_id -> Int4,
        device_code_autom -> Nullable<Text>,
    }
}

diesel::table! {
    units (id) {
        id -> Int4,
        client_id -> Int4,
        #[max_length = 250]
        unit_name -> Varchar,
        reference_id -> Int4,
        #[max_length = 100]
        city_name -> Nullable<Varchar>,
        #[max_length = 100]
        state_name -> Nullable<Varchar>,
        tarifa_kwh -> Nullable<Numeric>,
        constructed_area -> Nullable<Numeric>,
        capacity_power -> Nullable<Numeric>,
    }
}

diesel::table! {
    water_consumption_forecast (unit_id, forecast_date) {
        unit_id -> Int4,
        forecast_date -> Date,
        monday -> Nullable<Numeric>,
        tuesday -> Nullable<Numeric>,
        wednesday -> Nullable<Numeric>,
        thursday -> Nullable<Numeric>,
        friday -> Nullable<Numeric>,
        saturday -> Nullable<Numeric>,
        sunday -> Nullable<Numeric>,
    }
}

diesel::table! {
    water_hist (unit_id, record_date) {
        unit_id -> Int4,
        supplier -> Text,
        device_code -> Text,
        consumption -> Numeric,
        record_date -> Timestamp,
        is_measured_consumption -> Nullable<Bool>,
        is_valid_consumption -> Nullable<Bool>,
    }
}

diesel::table! {
    waters_hist (unit_id, record_date) {
        unit_id -> Int4,
        supplier -> Text,
        device_code -> Text,
        consumption -> Numeric,
        record_date -> Date,
    }
}

diesel::joinable!(assets -> units (unit_id));
diesel::joinable!(chiller_hx_parameters_minutes_hist -> units (unit_id));
diesel::joinable!(chiller_parameters_changes_hist -> units (unit_id));
diesel::joinable!(chiller_xa_hvar_parameters_minutes_hist -> units (unit_id));
diesel::joinable!(chiller_xa_parameters_minutes_hist -> units (unit_id));
diesel::joinable!(device_disponibility_hist -> units (unit_id));
diesel::joinable!(disponibility_hist -> units (unit_id));
diesel::joinable!(electric_circuits -> units (unit_id));
diesel::joinable!(energy_consumption_forecast -> electric_circuits (electric_circuit_id));
diesel::joinable!(energy_demand_minutes_hist -> electric_circuits (electric_circuit_id));
diesel::joinable!(energy_efficiency_hist -> machines (machine_id));
diesel::joinable!(energy_efficiency_hour_hist -> machines (machine_id));
diesel::joinable!(energy_hist -> electric_circuits (electric_circuit_id));
diesel::joinable!(energy_monthly_consumption_target -> units (unit_id));
diesel::joinable!(machines -> units (unit_id));
diesel::joinable!(units -> clients (client_id));
diesel::joinable!(water_consumption_forecast -> units (unit_id));
diesel::joinable!(water_hist -> units (unit_id));
diesel::joinable!(waters_hist -> units (unit_id));

diesel::allow_tables_to_appear_in_same_query!(
    assets,
    chiller_hx_parameters_minutes_hist,
    chiller_parameters_changes_hist,
    chiller_xa_hvar_parameters_minutes_hist,
    chiller_xa_parameters_minutes_hist,
    clients,
    device_disponibility_hist,
    devices_l1_totalization_hist,
    disponibility_hist,
    electric_circuits,
    energy_consumption_forecast,
    energy_demand_minutes_hist,
    energy_efficiency_hist,
    energy_efficiency_hour_hist,
    energy_hist,
    energy_monthly_consumption_target,
    last_device_telemetry_time,
    machines,
    units,
    water_consumption_forecast,
    water_hist,
    waters_hist,
);
