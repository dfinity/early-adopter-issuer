export const idlFactory = ({ IDL }) => {
  const IssuerConfig = IDL.Record({
    'derivation_origin' : IDL.Text,
    'idp_canister_ids' : IDL.Vec(IDL.Principal),
    'ic_root_key_der' : IDL.Vec(IDL.Nat8),
    'frontend_hostname' : IDL.Text,
  });
  const AddEventRequest = IDL.Record({
    'registration_code' : IDL.Opt(IDL.Text),
    'event_name' : IDL.Text,
  });
  const AddEventResponse = IDL.Record({
    'registration_code' : IDL.Text,
    'created_timestamp_s' : IDL.Nat32,
    'event_name' : IDL.Text,
  });
  const RegisterError = IDL.Variant({
    'Internal' : IDL.Text,
    'External' : IDL.Text,
  });
  const DerivationOriginRequest = IDL.Record({
    'frontend_hostname' : IDL.Text,
  });
  const DerivationOriginData = IDL.Record({ 'origin' : IDL.Text });
  const DerivationOriginError = IDL.Variant({
    'Internal' : IDL.Text,
    'UnsupportedOrigin' : IDL.Text,
  });
  const SignedIdAlias = IDL.Record({ 'credential_jws' : IDL.Text });
  const ArgumentValue = IDL.Variant({ 'Int' : IDL.Int32, 'String' : IDL.Text });
  const CredentialSpec = IDL.Record({
    'arguments' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, ArgumentValue))),
    'credential_type' : IDL.Text,
  });
  const GetCredentialRequest = IDL.Record({
    'signed_id_alias' : SignedIdAlias,
    'prepared_context' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'credential_spec' : CredentialSpec,
  });
  const IssuedCredentialData = IDL.Record({ 'vc_jws' : IDL.Text });
  const IssueCredentialError = IDL.Variant({
    'Internal' : IDL.Text,
    'SignatureNotFound' : IDL.Text,
    'InvalidIdAlias' : IDL.Text,
    'UnauthorizedSubject' : IDL.Text,
    'UnknownSubject' : IDL.Text,
    'UnsupportedCredentialSpec' : IDL.Text,
  });
  const HeaderField = IDL.Tuple(IDL.Text, IDL.Text);
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
    'certificate_version' : IDL.Opt(IDL.Nat16),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
    'status_code' : IDL.Nat16,
  });
  const EventRecord = IDL.Record({
    'registration_code' : IDL.Opt(IDL.Text),
    'created_timestamp_s' : IDL.Nat32,
    'event_name' : IDL.Text,
  });
  const ListEventsResponse = IDL.Record({ 'events' : IDL.Vec(EventRecord) });
  const PrepareCredentialRequest = IDL.Record({
    'signed_id_alias' : SignedIdAlias,
    'credential_spec' : CredentialSpec,
  });
  const PreparedCredentialData = IDL.Record({
    'prepared_context' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const RegisterUserEventData = IDL.Record({
    'registration_code' : IDL.Text,
    'event_name' : IDL.Text,
  });
  const RegisterUserRequest = IDL.Record({
    'event_data' : IDL.Opt(RegisterUserEventData),
  });
  const UserEventData = IDL.Record({
    'joined_timestamp_s' : IDL.Nat32,
    'event_name' : IDL.Text,
  });
  const EarlyAdopterResponse = IDL.Record({
    'joined_timestamp_s' : IDL.Nat32,
    'events' : IDL.Vec(UserEventData),
  });
  const Icrc21ConsentPreferences = IDL.Record({ 'language' : IDL.Text });
  const Icrc21VcConsentMessageRequest = IDL.Record({
    'preferences' : Icrc21ConsentPreferences,
    'credential_spec' : CredentialSpec,
  });
  const Icrc21ConsentInfo = IDL.Record({
    'consent_message' : IDL.Text,
    'language' : IDL.Text,
  });
  const Icrc21ErrorInfo = IDL.Record({ 'description' : IDL.Text });
  const Icrc21Error = IDL.Variant({
    'GenericError' : IDL.Record({
      'description' : IDL.Text,
      'error_code' : IDL.Nat,
    }),
    'UnsupportedCanisterCall' : Icrc21ErrorInfo,
    'ConsentMessageUnavailable' : Icrc21ErrorInfo,
  });
  return IDL.Service({
    'add_event' : IDL.Func(
        [AddEventRequest],
        [IDL.Variant({ 'Ok' : AddEventResponse, 'Err' : RegisterError })],
        [],
      ),
    'configure' : IDL.Func([IssuerConfig], [], []),
    'derivation_origin' : IDL.Func(
        [DerivationOriginRequest],
        [
          IDL.Variant({
            'Ok' : DerivationOriginData,
            'Err' : DerivationOriginError,
          }),
        ],
        [],
      ),
    'get_credential' : IDL.Func(
        [GetCredentialRequest],
        [
          IDL.Variant({
            'Ok' : IssuedCredentialData,
            'Err' : IssueCredentialError,
          }),
        ],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'list_events' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : ListEventsResponse, 'Err' : RegisterError })],
        [],
      ),
    'prepare_credential' : IDL.Func(
        [PrepareCredentialRequest],
        [
          IDL.Variant({
            'Ok' : PreparedCredentialData,
            'Err' : IssueCredentialError,
          }),
        ],
        [],
      ),
    'register_early_adopter' : IDL.Func(
        [RegisterUserRequest],
        [IDL.Variant({ 'Ok' : EarlyAdopterResponse, 'Err' : RegisterError })],
        [],
      ),
    'vc_consent_message' : IDL.Func(
        [Icrc21VcConsentMessageRequest],
        [IDL.Variant({ 'Ok' : Icrc21ConsentInfo, 'Err' : Icrc21Error })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const IssuerConfig = IDL.Record({
    'derivation_origin' : IDL.Text,
    'idp_canister_ids' : IDL.Vec(IDL.Principal),
    'ic_root_key_der' : IDL.Vec(IDL.Nat8),
    'frontend_hostname' : IDL.Text,
  });
  return [IDL.Opt(IssuerConfig)];
};
