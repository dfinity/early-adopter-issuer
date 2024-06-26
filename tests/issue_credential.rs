//! Tests related to issue_credential canister call.

use assert_matches::assert_matches;
use candid::{CandidType, Deserialize, Principal};
use canister_sig_util::{extract_raw_root_pk_from_der, CanisterSigPublicKey};
use canister_tests::api::http_request;
use canister_tests::api::internet_identity::vc_mvp as ii_api;
use canister_tests::flows;
use canister_tests::framework::{
    env, get_wasm_path, principal_1, principal_2, test_principal, time,
};
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_response_verification::types::VerificationInfo;
use ic_response_verification::verify_request_response_pair;
use ic_test_state_machine_client::{call_candid, call_candid_as, CanisterSettings};
use ic_test_state_machine_client::{query_candid_as, CallError, StateMachine};
use internet_identity_interface::http_gateway::{HttpRequest, HttpResponse};
use internet_identity_interface::internet_identity::types::vc_mvp::{
    GetIdAliasRequest, PrepareIdAliasRequest,
};
use internet_identity_interface::internet_identity::types::FrontendHostname;
use lazy_static::lazy_static;
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str;
use std::time::{Duration, UNIX_EPOCH};
use vc_util::issuer_api::{
    ArgumentValue, CredentialSpec, DerivationOriginData, DerivationOriginError,
    DerivationOriginRequest, GetCredentialRequest, Icrc21ConsentInfo, Icrc21ConsentPreferences,
    Icrc21Error, Icrc21VcConsentMessageRequest, IssueCredentialError, IssuedCredentialData,
    PrepareCredentialRequest, PreparedCredentialData, SignedIdAlias as SignedIssuerIdAlias,
};
use vc_util::{
    get_verified_id_alias_from_jws, validate_claims_match_spec,
    verify_credential_jws_with_canister_id,
};

const DUMMY_ROOT_KEY: &str ="308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100adf65638a53056b2222c91bb2457b0274bca95198a5acbdadfe7fd72178f069bdea8d99e9479d8087a2686fc81bf3c4b11fe275570d481f1698f79d468afe0e57acc1e298f8b69798da7a891bbec197093ec5f475909923d48bfed6843dbed1f";
const DUMMY_II_CANISTER_ID: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";

/// Dummy alias JWS for testing, valid wrt DUMMY_ROOT_KEY and DUMMY_II_CANISTER_ID.
/// id dapp: nugva-s7c6v-4yszt-koycv-5b623-an7q6-ha2nz-kz6rs-hawgl-nznbe-rqe
/// id alias: jkk22-zqdxc-kgpez-6sv2m-5pby4-wi4t2-prmoq-gf2ih-i2qtc-v37ac-5ae
const DUMMY_ALIAS_JWS: &str = "eyJqd2siOnsia3R5Ijoib2N0IiwiYWxnIjoiSWNDcyIsImsiOiJNRHd3REFZS0t3WUJCQUdEdUVNQkFnTXNBQW9BQUFBQUFBQUFBQUVCMGd6TTVJeXFMYUhyMDhtQTRWd2J5SmRxQTFyRVFUX2xNQnVVbmN5UDVVYyJ9LCJraWQiOiJkaWQ6aWNwOnJ3bGd0LWlpYWFhLWFhYWFhLWFhYWFhLWNhaSIsImFsZyI6IkljQ3MifQ.eyJleHAiOjE2MjAzMjk1MzAsImlzcyI6Imh0dHBzOi8vaWRlbnRpdHkuaWMwLmFwcC8iLCJuYmYiOjE2MjAzMjg2MzAsImp0aSI6ImRhdGE6dGV4dC9wbGFpbjtjaGFyc2V0PVVURi04LHRpbWVzdGFtcF9uczoxNjIwMzI4NjMwMDAwMDAwMDAwLGFsaWFzX2hhc2g6YTI3YzU4NTQ0MmUwN2RkZWFkZTRjNWE0YTAzMjdkMzA4NTE5NDAzYzRlYTM3NDIxNzBhZTRkYzk1YjIyZTQ3MyIsInN1YiI6ImRpZDppY3A6bnVndmEtczdjNnYtNHlzenQta295Y3YtNWI2MjMtYW43cTYtaGEybnota3o2cnMtaGF3Z2wtbnpuYmUtcnFlIiwidmMiOnsiQGNvbnRleHQiOiJodHRwczovL3d3dy53My5vcmcvMjAxOC9jcmVkZW50aWFscy92MSIsInR5cGUiOlsiVmVyaWZpYWJsZUNyZWRlbnRpYWwiLCJJbnRlcm5ldElkZW50aXR5SWRBbGlhcyJdLCJjcmVkZW50aWFsU3ViamVjdCI6eyJJbnRlcm5ldElkZW50aXR5SWRBbGlhcyI6eyJoYXNJZEFsaWFzIjoiamtrMjItenFkeGMta2dwZXotNnN2Mm0tNXBieTQtd2k0dDItcHJtb3EtZ2YyaWgtaTJxdGMtdjM3YWMtNWFlIn19fX0.2dn3omtjZXJ0aWZpY2F0ZVkBsdnZ96JkdHJlZYMBgwGDAYMCSGNhbmlzdGVygwGDAkoAAAAAAAAAAAEBgwGDAYMBgwJOY2VydGlmaWVkX2RhdGGCA1ggefxpZC6-L9hRQMZsK2uEkdp9i47qFyg05vdnXJdq5RaCBFgg0sz_P8xdqTDewOhKJUHmWFFrS7FQHnDotBDmmGoFfWCCBFggd489sLn21kn6CtdwO1z5LHQ4b4BzoMxL6iJ12AY71bWCBFggLCWMxIEbH-yaQBeRysmd_kWjQqRAViBeHSYLVZKLgt6CBFggFOLcHHMQti-VENiB1XWgOZCOTz4DpwkSY4F6Vvjzog2CBFggCeJanYMzwkFabt63nW6kpgqmwYesWVXAnhk3ziZxUG-CBFggczSkpyRrpe0-b3NqNLaT94GzICzbP_zSz-mePMBPWHaDAYIEWCA1U_ZYHVOz3Sdkb2HIsNoLDDiBuFfG3DxH6miIwRPra4MCRHRpbWWCA0mAuK7U3YmkvhZpc2lnbmF0dXJlWDCJnEu39Fhubo7bscFC63oGTZLQfGXoFKo5DS8m2O0Acsc_-gtngsZgnE7qCkT5yctkdHJlZYMBggRYIGX-wIQB1vS2FmgvqJPmB4qfbhugKsflrxhIMzunrun5gwJDc2lngwJYIIOQR7wl3Ws9Jb8VP4rhIb37XKLMkkZ2P7WaZ5we60WGgwGCBFgg21-OewBgqt_-0AtHHHS4yPyQK9g6JTHaGUuSIw4QYgqDAlgg5bQnHHvS3FfM_BaiSL6n19qoXkuA1KoLWk963fOUMW-CA0A";
const DUMMY_ALIAS_ID_DAPP_PRINCIPAL: &str =
    "nugva-s7c6v-4yszt-koycv-5b623-an7q6-ha2nz-kz6rs-hawgl-nznbe-rqe";

lazy_static! {
    /// Gzipped Wasm module for the current Early Adopter Issuer build, i.e. the one we're testing
    pub static ref EARLY_ADOPTER_ISSUER_WASM: Vec<u8> = {
        let def_path = PathBuf::from("./").join("early_adopter_issuer.wasm.gz");
        let err = format!("
        Could not find Early Adopter Issuer Wasm module for current build.
        I will look for it at {:?} (note that I run from {:?}).
        You can build the Wasm by running ./build.sh
        ", &def_path,
            &std::env::current_dir().map(|x| x.display().to_string()).unwrap_or_else(|_|
                "an unknown directory".to_string()));
                get_wasm_path("EARLY_ADOPTER_ISSUER_WASM".to_string(), &def_path).expect(&err)

    };

    pub static ref II_WASM: Vec<u8> = {
        let def_path = PathBuf::from("./").join("internet_identity.wasm.gz");
        let err = format!("
        Could not find Internet Identity Wasm module for current build.

        I will look for it at {:?}, and you can specify another path with the environment variable II_WASM (note that I run from {:?}).

        You can download the most recent II-wasm release from 
        https://github.com/dfinity/internet-identity/releases/latest/download/internet_identity_test.wasm.gz
        ", &def_path, &std::env::current_dir().map(|x| x.display().to_string()).unwrap_or_else(|_| "an unknown directory".to_string()));
        get_wasm_path("II_WASM".to_string(), &def_path).expect(&err)
    };

    pub static ref DUMMY_ISSUER_INIT: IssuerInit = IssuerInit::default();

    pub static ref DUMMY_SIGNED_ID_ALIAS: SignedIssuerIdAlias = SignedIssuerIdAlias {
        credential_jws: DUMMY_ALIAS_JWS.to_string(),
    };
}

pub fn install_canister(env: &StateMachine, wasm: Vec<u8>) -> CanisterId {
    let canister_id = env.create_canister(None);
    let arg = candid::encode_one("()").expect("error encoding issuer init arg as candid");
    env.install_canister(canister_id, wasm, arg, None);
    canister_id
}

#[derive(CandidType, Deserialize)]
pub struct IssuerInit {
    /// Root of trust for checking canister signatures.
    ic_root_key_der: Vec<u8>,
    /// List of canister ids that are allowed to provide id alias credentials.
    idp_canister_ids: Vec<Principal>,
    /// The derivation origin to be used by the issuer.
    derivation_origin: String,
    /// Frontend hostname to be used by the issuer.
    frontend_hostname: String,
}

impl Default for IssuerInit {
    fn default() -> Self {
        Self {
            ic_root_key_der: hex::decode(DUMMY_ROOT_KEY).unwrap(),
            idp_canister_ids: vec![Principal::from_text(DUMMY_II_CANISTER_ID).unwrap()],
            frontend_hostname: "https://default.host.name".to_string(),
            derivation_origin: "https://default.derivation.origin".to_string(),
        }
    }
}

#[derive(CandidType, Deserialize, Debug)]
pub struct EventData {
    pub event_name: String,
    pub registration_code: Option<String>,
    pub created_timestamp_s: u32,
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct UserEventData {
    pub joined_timestamp_s: u32,
    pub event_name: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct EarlyAdopterResponse {
    pub joined_timestamp_s: u32,
    pub events: Vec<UserEventData>,
}

#[derive(CandidType, Deserialize)]
pub struct ListEventsResponse {
    pub events: Vec<EventData>,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct RegisterUserEventData {
    pub event_name: String,
    pub registration_code: String,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct RegisterUserRequest {
    pub event_data: Option<RegisterUserEventData>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct AddEventResponse {
    pub event_name: String,
    pub registration_code: String,
    pub created_timestamp_s: u32,
}

#[derive(CandidType, Deserialize)]
pub struct AddEventRequest {
    pub event_name: String,
    pub registration_code: Option<String>,
}

#[derive(CandidType, Debug, Deserialize)]
pub enum EarlyAdopterError {
    Internal(String),
    External(String),
}

fn controller() -> Principal {
    Principal::self_authenticating("controller")
}

pub fn install_issuer(env: &StateMachine, init: &IssuerInit) -> CanisterId {
    let canister_controller = controller();
    let settings = CanisterSettings {
        controllers: Some(vec![canister_controller]),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
    };
    let canister_id = env.create_canister_with_settings(Some(settings), Some(canister_controller));
    let arg = candid::encode_one(Some(init)).expect("error encoding II installation arg as candid");
    env.install_canister(
        canister_id,
        EARLY_ADOPTER_ISSUER_WASM.clone(),
        arg,
        Some(canister_controller),
    );
    canister_id
}

mod api {
    use super::*;

    pub fn configure(
        env: &StateMachine,
        canister_id: CanisterId,
        config: &IssuerInit,
    ) -> Result<(), CallError> {
        call_candid(env, canister_id, "configure", (config,))
    }

    pub fn vc_consent_message(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        consent_message_request: &Icrc21VcConsentMessageRequest,
    ) -> Result<Result<Icrc21ConsentInfo, Icrc21Error>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "vc_consent_message",
            (consent_message_request,),
        )
        .map(|(x,)| x)
    }

    pub fn derivation_origin(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        derivation_origin_req: &DerivationOriginRequest,
    ) -> Result<Result<DerivationOriginData, DerivationOriginError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "derivation_origin",
            (derivation_origin_req,),
        )
        .map(|(x,)| x)
    }

    pub fn add_event(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        request: &AddEventRequest,
    ) -> Result<Result<AddEventResponse, EarlyAdopterError>, CallError> {
        call_candid_as(env, canister_id, sender, "add_event", (request,)).map(|(x,)| x)
    }

    pub fn list_events(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
    ) -> Result<Result<ListEventsResponse, EarlyAdopterError>, CallError> {
        call_candid_as(env, canister_id, sender, "list_events", ()).map(|(x,)| x)
    }

    pub fn register_early_adopter(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        request: &RegisterUserRequest,
    ) -> Result<Result<EarlyAdopterResponse, EarlyAdopterError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "register_early_adopter",
            (request,),
        )
        .map(|(x,)| x)
    }

    pub fn prepare_credential(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        prepare_credential_request: &PrepareCredentialRequest,
    ) -> Result<Result<PreparedCredentialData, IssueCredentialError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "prepare_credential",
            (prepare_credential_request,),
        )
        .map(|(x,)| x)
    }

    pub fn get_credential(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        get_credential_request: &GetCredentialRequest,
    ) -> Result<Result<IssuedCredentialData, IssueCredentialError>, CallError> {
        query_candid_as(
            env,
            canister_id,
            sender,
            "get_credential",
            (get_credential_request,),
        )
        .map(|(x,)| x)
    }
}

#[test]
fn should_get_vc_consent_message_for_eary_adopter() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: early_adopter_credential_spec(),
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let consent_info =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed")
            .expect("Failed to obtain consent info");
    assert!(consent_info
        .consent_message
        .contains("You became an early adopter"));
}

#[test]
fn should_get_vc_consent_message_for_event_attendance() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    let event_name = "DICE2024".to_string();
    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: event_attendance_credential_spec(event_name),
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let consent_info =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed")
            .expect("Failed to obtain consent info");
    assert!(consent_info
        .consent_message
        .contains("You have attended the event"));
}

#[test]
fn should_fail_vc_consent_message_if_not_supported() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: CredentialSpec {
            credential_type: "VerifiedResident".to_string(),
            arguments: None,
        },
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let response =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed");
    assert_matches!(response, Err(Icrc21Error::ConsentMessageUnavailable(_)));
}

#[test]
fn should_fail_early_adopter_vc_consent_message_if_missing_arguments() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: CredentialSpec {
            credential_type: "EarlyAdopter".to_string(),
            arguments: None,
        },
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let response =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed");
    assert_matches!(response, Err(Icrc21Error::ConsentMessageUnavailable(_)));
}

#[test]
fn should_fail_event_attendance_vc_consent_message_if_missing_arguments() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: CredentialSpec {
            credential_type: "EventAttendance".to_string(),
            arguments: None,
        },
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let response =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed");
    assert_matches!(response, Err(Icrc21Error::ConsentMessageUnavailable(_)));
}

#[test]
fn should_fail_early_adopter_vc_consent_message_if_missing_required_argument() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    let mut args = HashMap::new();
    args.insert("wrongArgument".to_string(), ArgumentValue::Int(42));

    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: CredentialSpec {
            credential_type: "EarlyAdopter".to_string(),
            arguments: Some(args),
        },
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let response =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed");
    assert_matches!(response, Err(Icrc21Error::ConsentMessageUnavailable(_)));
}

#[test]
fn should_fail_event_attendance_vc_consent_message_if_missing_required_argument() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    let mut args = HashMap::new();
    args.insert("wrongArgument".to_string(), ArgumentValue::Int(42));

    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: CredentialSpec {
            credential_type: "EventAttendance".to_string(),
            arguments: Some(args),
        },
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let response =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed");
    assert_matches!(response, Err(Icrc21Error::ConsentMessageUnavailable(_)));
}

#[test]
fn should_not_return_derivation_origin_if_not_in_config() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let frontend_hostname = format!("https://{}.icp0.io", canister_id.to_text());
    let req = DerivationOriginRequest { frontend_hostname };
    match api::derivation_origin(&env, canister_id, principal_1(), &req).unwrap() {
        Ok(_) => assert!(false),
        Err(DerivationOriginError::UnsupportedOrigin(_)) => assert!(true),
        Err(_) => assert!(false),
    }
}

#[test]
fn should_return_derivation_origin_with_custom_init() {
    let env = env();
    let custom_init = IssuerInit {
        ic_root_key_der: hex::decode(DUMMY_ROOT_KEY).unwrap(),
        idp_canister_ids: vec![Principal::from_text(DUMMY_II_CANISTER_ID).unwrap()],
        derivation_origin: "https://custom.derivation_origin".to_string(),
        frontend_hostname: "https://custom.frontend.host.name".to_string(),
    };
    let canister_id = install_issuer(&env, &custom_init);
    let response = api::derivation_origin(
        &env,
        canister_id,
        principal_1(),
        &DerivationOriginRequest {
            frontend_hostname: custom_init.frontend_hostname.clone(),
        },
    )
    .expect("API call failed")
    .expect("derivation_origin error");
    assert_eq!(response.origin, custom_init.derivation_origin);
}

#[test]
fn should_fail_derivation_origin_if_unsupported_origin() {
    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let req = DerivationOriginRequest {
        frontend_hostname: "https://wrong.fe.host".to_string(),
    };
    let response =
        api::derivation_origin(&env, canister_id, principal_1(), &req).expect("API call failed");
    assert_eq!(
        response,
        Err(DerivationOriginError::UnsupportedOrigin(
            req.frontend_hostname
        ))
    );
}

fn early_adopter_credential_spec() -> CredentialSpec {
    let mut args = HashMap::new();
    args.insert("sinceYear".to_string(), ArgumentValue::Int(2024));
    CredentialSpec {
        credential_type: "EarlyAdopter".to_string(),
        arguments: Some(args),
    }
}

fn event_attendance_credential_spec(event_name: String) -> CredentialSpec {
    let mut args = HashMap::new();
    args.insert("eventName".to_string(), ArgumentValue::String(event_name));
    CredentialSpec {
        credential_type: "EventAttendance".to_string(),
        arguments: Some(args),
    }
}

#[test]
fn should_fail_prepare_credential_for_unauthorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap(),
        &PrepareCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Err(e) if format!("{:?}", e).contains("unregistered principal"));
}

#[test]
fn should_fail_prepare_credential_for_wrong_sender() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let signed_id_alias = DUMMY_SIGNED_ID_ALIAS.clone();

    let response = api::prepare_credential(
        &env,
        issuer_id,
        principal_1(), // not the same as contained in signed_id_alias
        &PrepareCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias,
        },
    )
    .expect("API call failed");
    assert_matches!(response,
        Err(IssueCredentialError::InvalidIdAlias(e)) if e.contains("id alias could not be verified")
    );
}

#[test]
fn should_fail_get_credential_for_wrong_sender() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let signed_id_alias = DUMMY_SIGNED_ID_ALIAS.clone();
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    let request = RegisterUserRequest { event_data: None };
    let _ = api::register_early_adopter(&env, issuer_id, authorized_principal, &request).unwrap();
    let unauthorized_principal = test_principal(2);

    let prepare_credential_response = api::prepare_credential(
        &env,
        issuer_id,
        authorized_principal,
        &PrepareCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias: signed_id_alias.clone(),
        },
    )
    .expect("API call failed")
    .expect("failed to prepare credential");

    let get_credential_response = api::get_credential(
        &env,
        issuer_id,
        unauthorized_principal,
        &GetCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias,
            prepared_context: prepare_credential_response.prepared_context,
        },
    )
    .expect("API call failed");
    assert_matches!(get_credential_response,
        Err(IssueCredentialError::InvalidIdAlias(e)) if e.contains("id alias could not be verified")
    );
}

#[test]
fn should_fail_prepare_credential_for_anonymous_caller() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::anonymous(),
        &PrepareCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response,
        Err(IssueCredentialError::InvalidIdAlias(e)) if e.contains("id alias could not be verified")
    );
}

#[test]
fn should_fail_prepare_credential_for_wrong_root_key() {
    let env = env();
    let issuer_id = install_issuer(
        &env,
        &IssuerInit {
            ic_root_key_der: canister_sig_util::IC_ROOT_PK_DER.to_vec(), // does not match the DUMMY_ROOT_KEY, which is used in DUMMY_ALIAS_JWS
            ..IssuerInit::default()
        },
    );
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap(),
        &PrepareCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Err(IssueCredentialError::InvalidIdAlias(_)));
}

#[test]
fn should_fail_prepare_credential_for_wrong_idp_canister_id() {
    let env = env();
    let issuer_id = install_issuer(
        &env,
        &IssuerInit {
            idp_canister_ids: vec![Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap()], // does not match the DUMMY_II_CANISTER_ID, which is used in DUMMY_ALIAS_JWS
            ..IssuerInit::default()
        },
    );
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap(),
        &PrepareCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Err(IssueCredentialError::InvalidIdAlias(_)));
}

#[test]
fn should_prepare_early_adopter_credential_for_authorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    let request = RegisterUserRequest { event_data: None };
    let _ = api::register_early_adopter(&env, issuer_id, authorized_principal, &request).unwrap();
    let response = api::prepare_credential(
        &env,
        issuer_id,
        authorized_principal,
        &PrepareCredentialRequest {
            credential_spec: early_adopter_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Ok(_));
}

#[test]
fn should_fail_to_prepare_event_attendance_credential_for_non_attendees() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    let attended_event = "DICE2024".to_string();
    let attended_event_code = "code".to_string();
    let not_attended_event = "Denver2025".to_string();
    let event_request = AddEventRequest {
        event_name: attended_event.clone(),
        registration_code: Some(attended_event_code.clone()),
    };
    let _ = api::add_event(&env, issuer_id, controller(), &event_request).unwrap();
    let event_data = RegisterUserEventData {
        event_name: attended_event.clone(),
        registration_code: attended_event_code.clone(),
    };
    let request = RegisterUserRequest {
        event_data: Some(event_data),
    };
    let status_user = api::register_early_adopter(&env, issuer_id, authorized_principal, &request)
        .unwrap()
        .unwrap();
    assert_eq!(status_user.events.len(), 1);
    let response = api::prepare_credential(
        &env,
        issuer_id,
        authorized_principal,
        &PrepareCredentialRequest {
            credential_spec: event_attendance_credential_spec(not_attended_event.clone()),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Err(IssueCredentialError::UnauthorizedSubject(_)));
}

#[test]
fn should_prepare_event_attendance_credential_for_attendees() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    let event_name = "DICE2024".to_string();
    let event_code = "code".to_string();
    let event_request = AddEventRequest {
        event_name: event_name.clone(),
        registration_code: Some(event_code.clone()),
    };
    let _ = api::add_event(&env, issuer_id, controller(), &event_request).unwrap();
    let event_data = RegisterUserEventData {
        event_name: event_name.clone(),
        registration_code: event_code.clone(),
    };
    let request = RegisterUserRequest {
        event_data: Some(event_data),
    };
    let status_user = api::register_early_adopter(&env, issuer_id, authorized_principal, &request)
        .unwrap()
        .unwrap();
    assert_eq!(status_user.events.len(), 1);
    let response = api::prepare_credential(
        &env,
        issuer_id,
        authorized_principal,
        &PrepareCredentialRequest {
            credential_spec: event_attendance_credential_spec(event_name.clone()),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Ok(_));
}

/// Verifies that different credentials are being created including II interactions.
#[test]
fn should_issue_credential_e2e() -> Result<(), CallError> {
    let env = env();
    let ii_id = install_canister(&env, II_WASM.clone());
    let issuer_id = install_issuer(
        &env,
        &IssuerInit {
            ic_root_key_der: env.root_key().to_vec(),
            idp_canister_ids: vec![ii_id],
            ..IssuerInit::default()
        },
    );
    let identity_number = flows::register_anchor(&env, ii_id);
    let relying_party = FrontendHostname::from("https://some-dapp.com");
    let issuer = FrontendHostname::from("https://some-issuer.com");

    let prepare_id_alias_req = PrepareIdAliasRequest {
        identity_number,
        relying_party: relying_party.clone(),
        issuer: issuer.clone(),
    };

    let prepared_id_alias =
        ii_api::prepare_id_alias(&env, ii_id, principal_1(), prepare_id_alias_req)?
            .expect("prepare id_alias failed");

    let canister_sig_pk =
        CanisterSigPublicKey::try_from(prepared_id_alias.canister_sig_pk_der.as_ref())
            .expect("failed parsing canister sig pk");

    let get_id_alias_req = GetIdAliasRequest {
        identity_number,
        relying_party,
        issuer,
        rp_id_alias_jwt: prepared_id_alias.rp_id_alias_jwt,
        issuer_id_alias_jwt: prepared_id_alias.issuer_id_alias_jwt,
    };
    let id_alias_credentials = ii_api::get_id_alias(&env, ii_id, principal_1(), get_id_alias_req)?
        .expect("get id_alias failed");

    let root_pk_raw =
        extract_raw_root_pk_from_der(&env.root_key()).expect("Failed decoding IC root key.");
    let alias_tuple = get_verified_id_alias_from_jws(
        &id_alias_credentials
            .issuer_id_alias_credential
            .credential_jws,
        &id_alias_credentials.issuer_id_alias_credential.id_dapp,
        &canister_sig_pk.canister_id,
        &root_pk_raw,
        env.time().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
    )
    .expect("Invalid ID alias");

    let event_name = "DICE2024".to_string();
    let event_code = "code".to_string();
    let event_request = AddEventRequest {
        event_name: event_name.clone(),
        registration_code: Some(event_code.clone()),
    };
    let _ = api::add_event(&env, issuer_id, controller(), &event_request).unwrap();
    let event_data = RegisterUserEventData {
        event_name: event_name.clone(),
        registration_code: event_code.clone(),
    };
    let request = RegisterUserRequest {
        event_data: Some(event_data),
    };
    let _ = api::register_early_adopter(&env, issuer_id, alias_tuple.id_dapp, &request)?;

    for credential_spec in [
        early_adopter_credential_spec(),
        event_attendance_credential_spec(event_name.clone()),
    ] {
        let early_adopter_prepared_credential = api::prepare_credential(
            &env,
            issuer_id,
            id_alias_credentials.issuer_id_alias_credential.id_dapp,
            &PrepareCredentialRequest {
                credential_spec: credential_spec.clone(),
                signed_id_alias: SignedIssuerIdAlias {
                    credential_jws: id_alias_credentials
                        .issuer_id_alias_credential
                        .credential_jws
                        .clone(),
                },
            },
        )?
        .expect("failed to prepare credential");

        let early_adopter_get_credential_response = api::get_credential(
            &env,
            issuer_id,
            id_alias_credentials.issuer_id_alias_credential.id_dapp,
            &GetCredentialRequest {
                credential_spec: credential_spec.clone(),
                signed_id_alias: SignedIssuerIdAlias {
                    credential_jws: id_alias_credentials
                        .issuer_id_alias_credential
                        .credential_jws
                        .clone(),
                },
                prepared_context: early_adopter_prepared_credential.prepared_context,
            },
        )?;
        let early_adopter_claims = verify_credential_jws_with_canister_id(
            &early_adopter_get_credential_response.unwrap().vc_jws,
            &issuer_id,
            &root_pk_raw,
            env.time().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        )
        .expect("credential verification failed");
        let early_adopter_vc_claims = early_adopter_claims.vc().expect("missing VC claims");
        validate_claims_match_spec(early_adopter_vc_claims, &credential_spec)
            .expect("Claim validation failed");
    }

    Ok(())
}

#[test]
fn should_configure() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    api::configure(&env, issuer_id, &DUMMY_ISSUER_INIT).expect("API call failed");
}

/// Verifies that the expected assets is delivered and certified.
#[test]
fn issuer_canister_serves_http_assets() -> Result<(), CallError> {
    fn verify_response_certification(
        env: &StateMachine,
        canister_id: CanisterId,
        request: HttpRequest,
        http_response: HttpResponse,
        min_certification_version: u16,
    ) -> VerificationInfo {
        verify_request_response_pair(
            ic_http_certification::HttpRequest {
                method: request.method,
                url: request.url,
                headers: request.headers,
                body: request.body.into_vec(),
            },
            ic_http_certification::HttpResponse {
                status_code: http_response.status_code,
                headers: http_response.headers,
                body: http_response.body.into_vec(),
                upgrade: None,
            },
            canister_id.as_slice(),
            time(env) as u128,
            Duration::from_secs(300).as_nanos(),
            &env.root_key(),
            min_certification_version as u8,
        )
        .unwrap_or_else(|e| panic!("validation failed: {e}"))
    }

    let env = env();
    let canister_id = install_issuer(&env, &DUMMY_ISSUER_INIT);

    // for each asset and certification version, fetch the asset, check the HTTP status code, headers and certificate.

    for certification_version in 1..=2 {
        let request = HttpRequest {
            method: "GET".to_string(),
            url: "/".to_string(),
            headers: vec![],
            body: ByteBuf::new(),
            certificate_version: Some(certification_version),
        };
        let http_response = http_request(&env, canister_id, &request)?;
        assert_eq!(http_response.status_code, 200);

        let result = verify_response_certification(
            &env,
            canister_id,
            request,
            http_response,
            certification_version,
        );
        assert_eq!(result.verification_version, certification_version);
    }

    Ok(())
}

/// Verifies that the expected assets is delivered and certified.
#[test]
fn issuer_canister_serves_metrics_endpoint() -> Result<(), CallError> {
    fn assert_metrics(
        env: &StateMachine,
        canister_id: Principal,
        expected_substring: &str,
    ) -> Result<(), CallError> {
        let request = HttpRequest {
            method: "GET".to_string(),
            url: "/metrics".to_string(),
            headers: vec![],
            body: ByteBuf::new(),
            certificate_version: Some(1),
        };
        let http_response = http_request(&env, canister_id, &request)?;
        assert_eq!(http_response.status_code, 200);

        match str::from_utf8(&http_response.body) {
            Ok(metrics_str) => {
                print!("{}", metrics_str.to_string());
                assert!(metrics_str.contains(expected_substring));
            }
            Err(_) => {
                assert!(false);
            }
        };

        Ok(())
    }

    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let request = RegisterUserRequest { event_data: None };

    assert_metrics(&env, issuer_id, "early_adopters 0")?;

    api::register_early_adopter(&env, issuer_id, principal_1(), &request)?
        .expect("Failed registering user");

    assert_metrics(&env, issuer_id, "early_adopters 1")?;

    env.advance_time(std::time::Duration::from_secs(2));

    api::register_early_adopter(&env, issuer_id, principal_2(), &request)?
        .expect("Failed registering user");

    env.advance_time(std::time::Duration::from_secs(2));

    assert_metrics(&env, issuer_id, "early_adopters 2")?;

    Ok(())
}

#[test]
fn should_not_overwrite_the_first_registration() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let user_a = principal_1();
    let user_b = principal_2();
    let request = RegisterUserRequest { event_data: None };

    // Register two users at differen time, they should have different timestamps
    let status_1_user_a = api::register_early_adopter(&env, issuer_id, user_a, &request)?
        .expect("Failed registering user a");
    env.advance_time(std::time::Duration::from_secs(2));
    let status_1_user_b = api::register_early_adopter(&env, issuer_id, user_b, &request)?
        .expect("Failed registering user b");
    assert_ne!(
        status_1_user_a.joined_timestamp_s,
        status_1_user_b.joined_timestamp_s
    );

    // Re-register user a, it's timestamp should not change
    let status_2_user_a = api::register_early_adopter(&env, issuer_id, user_a, &request)?
        .expect("Failed getting status for user a");
    assert_eq!(
        status_1_user_a.joined_timestamp_s,
        status_2_user_a.joined_timestamp_s
    );

    // Re-register user a again, it's timestamp should still not change
    env.advance_time(std::time::Duration::from_secs(2));
    let status_3_user_a = api::register_early_adopter(&env, issuer_id, user_a, &request)?
        .expect("Failed getting status for user a");
    assert_eq!(
        status_1_user_a.joined_timestamp_s,
        status_3_user_a.joined_timestamp_s
    );

    // Re-register user b, it's timestamp should not change
    let status_2_user_b = api::register_early_adopter(&env, issuer_id, user_b, &request)?
        .expect("Failed getting status for user b");
    assert_eq!(
        status_1_user_b.joined_timestamp_s,
        status_2_user_b.joined_timestamp_s
    );

    Ok(())
}

#[test]
fn should_add_events_to_registered_user() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let user = principal_1();
    let event_name_a = "event A".to_string();
    let event_code_a = "code A".to_string();
    let event_name_b = "event B".to_string();
    let event_code_b = "code A".to_string();
    let event_data_a = RegisterUserEventData {
        event_name: event_name_a.clone(),
        registration_code: event_code_a.clone(),
    };
    let event_data_b = RegisterUserEventData {
        event_name: event_name_b.clone(),
        registration_code: event_code_b.clone(),
    };
    let event_request_a = AddEventRequest {
        event_name: event_name_a.clone(),
        registration_code: Some(event_code_a.clone()),
    };
    let _ = api::add_event(&env, issuer_id, controller(), &event_request_a).unwrap();
    let event_request_b = AddEventRequest {
        event_name: event_name_b.clone(),
        registration_code: Some(event_code_b.clone()),
    };
    let _ = api::add_event(&env, issuer_id, controller(), &event_request_b).unwrap();

    let request_a = RegisterUserRequest {
        event_data: Some(event_data_a),
    };
    let request_b = RegisterUserRequest {
        event_data: Some(event_data_b),
    };
    // Register with event_a
    let status_1_user_a = api::register_early_adopter(&env, issuer_id, user, &request_a)?
        .expect("Failed registering user a");
    assert_eq!(status_1_user_a.events.len(), 1);

    env.advance_time(std::time::Duration::from_secs(2));

    // Re-register user a with event_b adds event but doesn't change timestamp
    let status_2_user_a = api::register_early_adopter(&env, issuer_id, user, &request_b)?
        .expect("Failed getting status for user a");
    assert_eq!(
        status_1_user_a.joined_timestamp_s,
        status_2_user_a.joined_timestamp_s
    );
    assert_eq!(status_2_user_a.events.len(), 2);

    Ok(())
}

#[test]
fn should_fail_to_register_user_with_empty_event_name() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let user = principal_1();
    let event_data = RegisterUserEventData {
        event_name: "".to_string(),
        registration_code: "".to_string(),
    };
    let empty_event = RegisterUserRequest {
        event_data: Some(event_data),
    };

    let status_1_user =
        api::register_early_adopter(&env, issuer_id, user, &empty_event)?.unwrap_err();

    match status_1_user {
        EarlyAdopterError::External(msg) => assert!(msg.contains("cannot be an empty string")),
        _ => assert!(false),
    }

    Ok(())
}

#[test]
fn only_controllers_can_add_events() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let user = principal_1();
    let empty_event = AddEventRequest {
        event_name: "Test".to_string(),
        registration_code: None,
    };

    let response = api::add_event(&env, issuer_id, user, &empty_event)?.unwrap_err();

    match response {
        EarlyAdopterError::External(msg) => {
            assert!(msg.contains("Only controllers can register events"))
        }
        _ => assert!(false),
    }

    Ok(())
}

#[test]
fn should_add_event_with_random_code() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let empty_event = AddEventRequest {
        event_name: "Test".to_string(),
        registration_code: None,
    };

    api::add_event(&env, issuer_id, controller(), &empty_event)?
        .expect("API call to register event failed");

    let events_response =
        api::list_events(&env, issuer_id, controller())?.expect("API to list events failed");

    assert!(events_response.events.len() == 1);
    assert_matches!(events_response.events[0].registration_code, Some(_));

    Ok(())
}

#[test]
fn should_add_event_with_code() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let code = "code".to_string();
    let empty_event = AddEventRequest {
        event_name: "Test".to_string(),
        registration_code: Some(code.clone()),
    };

    api::add_event(&env, issuer_id, controller(), &empty_event)?.expect("API call failed");

    let events_response =
        api::list_events(&env, issuer_id, controller())?.expect("API to list events failed");

    assert!(events_response.events.len() == 1);
    assert_eq!(
        events_response.events[0].registration_code.clone().unwrap(),
        code
    );

    Ok(())
}

#[test]
fn should_not_register_same_event_name_twice() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let code = "code".to_string();
    let empty_event = AddEventRequest {
        event_name: "Test".to_string(),
        registration_code: Some(code.clone()),
    };

    api::add_event(&env, issuer_id, controller(), &empty_event)?.expect("API call failed");
    let register_response =
        api::add_event(&env, issuer_id, controller(), &empty_event)?.unwrap_err();

    match register_response {
        EarlyAdopterError::External(msg) => {
            assert!(msg.contains("already exists"))
        }
        _ => assert!(false),
    }

    Ok(())
}

#[test]
fn should_add_events() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let event_1 = AddEventRequest {
        event_name: "Test 1".to_string(),
        registration_code: None,
    };
    let event_2 = AddEventRequest {
        event_name: "Test 2".to_string(),
        registration_code: None,
    };
    let event_3 = AddEventRequest {
        event_name: "Test 3".to_string(),
        registration_code: None,
    };

    api::add_event(&env, issuer_id, controller(), &event_1)?.expect("API call failed");
    api::add_event(&env, issuer_id, controller(), &event_2)?.expect("API call failed");
    api::add_event(&env, issuer_id, controller(), &event_3)?.expect("API call failed");

    let events_response =
        api::list_events(&env, issuer_id, controller())?.expect("API to list events failed");

    assert!(events_response.events.len() == 3);

    Ok(())
}

#[test]
fn should_upgrade_issuer() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let arg = candid::encode_one("()").expect("error encoding issuer init arg as candid");
    env.upgrade_canister(
        issuer_id,
        EARLY_ADOPTER_ISSUER_WASM.clone(),
        arg,
        Some(controller()),
    )?;

    // Verify the canister is still running.
    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: early_adopter_credential_spec(),
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };
    let _ = api::vc_consent_message(&env, issuer_id, principal_1(), &consent_message_request)
        .expect("API call failed")
        .expect("Failed to obtain consent info");
    Ok(())
}

#[test]
fn should_retain_adopters_after_upgrade() -> Result<(), CallError> {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let request = RegisterUserRequest { event_data: None };
    let status_before = api::register_early_adopter(&env, issuer_id, principal_1(), &request)?
        .expect("Failed registering");
    let arg = candid::encode_one("()").expect("error encoding issuer init arg as candid");
    env.upgrade_canister(
        issuer_id,
        EARLY_ADOPTER_ISSUER_WASM.clone(),
        arg,
        Some(controller()),
    )?;
    env.advance_time(std::time::Duration::from_secs(2));
    let status_for_new_user =
        api::register_early_adopter(&env, issuer_id, principal_2(), &request)?
            .expect("Failed registering new user");
    assert_ne!(
        status_before.joined_timestamp_s,
        status_for_new_user.joined_timestamp_s
    );
    let status_after = api::register_early_adopter(&env, issuer_id, principal_1(), &request)?
        .expect("Failed getting status");
    assert_eq!(
        status_before.joined_timestamp_s,
        status_after.joined_timestamp_s
    );

    let status_after_repeated =
        api::register_early_adopter(&env, issuer_id, principal_1(), &request)?
            .expect("Failed getting status");
    assert_eq!(
        status_before.joined_timestamp_s,
        status_after_repeated.joined_timestamp_s
    );
    Ok(())
}
