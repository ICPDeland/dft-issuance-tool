use crate::types::*;
use ic_cdk::export::candid::{CandidType, Nat, Principal};
use ic_cdk::{api, caller, id, storage};
use ic_cdk_macros::*;
use serde::Deserialize;
use serde_bytes::{ByteBuf, Bytes};

pub async fn create_canister_call(args: CreateCanisterArgs) -> Result<CreateResult, String> {
    #[derive(CandidType)]
    struct In {
        settings: Option<CanisterSettings>,
    }
    let in_arg = In {
        settings: Some(args.settings),
    };

    let (create_result,): (CreateResult,) = match api::call::call_with_payment(
        Principal::management_canister(),
        "create_canister",
        (in_arg,),
        args.cycles,
    )
    .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ))
        }
    };

    Ok(create_result)
}

async fn update_settings_call(args: UpdateSettingsArgs) -> Result<(), String> {
    match api::call::call(Principal::management_canister(), "update_settings", (args,)).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ))
        }
    };
    Ok(())
}

pub async fn install_canister(
    canister_id: &Principal,
    wasm_module: Vec<u8>,
    args: Vec<u8>,
) -> Result<(), String> {
    // Install Wasm
    #[derive(CandidType, Deserialize)]
    enum InstallMode {
        #[serde(rename = "install")]
        Install,
        #[serde(rename = "reinstall")]
        Reinstall,
        #[serde(rename = "upgrade")]
        Upgrade,
    }

    #[derive(CandidType, Deserialize)]
    struct CanisterInstall {
        mode: InstallMode,
        canister_id: Principal,
        #[serde(with = "serde_bytes")]
        wasm_module: Vec<u8>,
        #[serde(with = "serde_bytes")]
        arg: Vec<u8>,
    }

    let install_config = CanisterInstall {
        mode: InstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: wasm_module.clone(),
        arg: args,
    };

    match api::call::call(
        Principal::management_canister(),
        "install_code",
        (install_config,),
    )
    .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ))
        }
    };

    // Store wallet wasm
    let store_args = TokenStoreWASMArgs { wasm_module };
    match api::call::call(
        canister_id.clone(),
        "wallet_store_wallet_wasm",
        (store_args,),
    )
    .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ))
        }
    };
    Ok(())
}
