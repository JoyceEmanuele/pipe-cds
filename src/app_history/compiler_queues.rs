use crate::app_history::{ dma_hist, energy_hist, dut_hist, dac_hist, dri_hist, dmt_hist, dal_hist, dam_hist };
use crate::GlobalVars;
use std::error::Error;
use std::sync::Arc;

#[derive(Debug)]
pub enum CompilationRequest {
    CompDma(dma_hist::ReqParameters),
    EnergyQuery(energy_hist::EnergyHistParams),
    CompDut(dut_hist::ReqParameters),
    CompDac(dac_hist::ReqParameters),
    CompDri(dri_hist::DriHistParams),
    CompDmt(dmt_hist::ReqParameters),
    CompDal(dal_hist::ReqParameters),
    CompDam(dam_hist::ReqParameters)
}

pub async fn task_queue_manager(request: CompilationRequest, globs: &Arc<GlobalVars>) -> Result<String, Box<dyn Error>>{
    let response = match executar_requisicao(request, globs).await {
        Ok(v) => v,
        Err(err) => return Err(format!("Erro ao executar requisição, {}", err).into()),
    };

    Ok(response)
}

async fn executar_requisicao(
    request: CompilationRequest,
    globs: &Arc<GlobalVars>,
) -> Result<String, Box<dyn Error>> {
    match request {
        CompilationRequest::CompDac(body) => { dac_hist::process_comp_command_dac_v2(body, globs).await }
        CompilationRequest::CompDma(body) => { dma_hist::process_comp_command_dma(body, globs).await },
        CompilationRequest::CompDut(body) => { dut_hist::process_comp_command_dut(body, globs).await },
        CompilationRequest::CompDri(body) => { body.process_query(globs).await },
        CompilationRequest::EnergyQuery(body) => { body.process_query(globs).await },
        CompilationRequest::CompDmt(body) => { dmt_hist::process_comp_command_dmt(body, globs).await },
        CompilationRequest::CompDal(body) => { dal_hist::process_comp_command_dal(body, globs).await },
        CompilationRequest::CompDam(body) => { dam_hist::process_comp_command_dam(body, globs).await },
    }
}
