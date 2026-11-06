# Identity spoofing — architecture and feasibility

Status: partially implemented. Identity schema, generator, WebAssembly bridge, WebUI, and a narrow app-local `Build.SERIAL` field override are present. `Build.getSerial()` and all other identifier APIs remain unmodified.
Target: Device Faker's existing Zygisk-only module flow, API 29–35, arm64 first.

## 1. Goals and non-goals

The identity layer is opt-in and process-scoped. It must not alter CPU, SoC/chipset, GPU, ABI, display metrics, sensors, kernel data, or other hardware characteristics. The existing `cpu_spoof` path is out of scope and remains independent.

The module is a single Magisk/KernelSU/APatch ZIP loaded through the repository's existing Zygisk `on_load`, `pre_app_specialize`, and `post_app_specialize` lifecycle. It must not depend on LSPosed, Xposed, Riru, or a separate manager.

The current repository calls `hook_build_fields` during `pre_app_specialize`, applies per-process COW property changes, then requests `DlCloseModuleLibrary`. Any persistent native hook therefore changes the current lifetime model and must be justified and isolated.

## 2. Existing architecture facts

- `src/lib.rs` extracts the app package and Android user ID in `pre_app_specialize`, loads `/data/adb/device_faker/config/config.toml`, resolves the app/template config, calls `hook_build_fields`, applies COW property changes, and requests `DlCloseModuleLibrary`.
- Server specialization immediately requests `DlCloseModuleLibrary`; no system_server hook is part of the current design.
- `src/config.rs` uses Serde structs, `IndexMap` for deterministic template order, `[[apps]]` lookup before template lookup for the same resolved package key, and package keys supporting `@userId`.
- The existing TOML parser is `toml` (not `toml_edit`) in the module crate. The CLI is a separate crate and is not currently part of the module ZIP.
- `xtask` stages `module/`, builds WebUI, and copies only the ARM64 Zygisk shared object into `module/zygisk/arm64-v8a.so`. `module/customize.sh` handles installation and config migration.
- WebUI edits config in the browser through its config store; a shared generator should live in Rust only if there is a real runtime/backend boundary that invokes it. A pure static Vue UI cannot call arbitrary Rust code in the phone's app process without adding a companion/CLI IPC interface.

These facts make a direct ART-hook implementation materially different from current JNI field replacement.

## 3. Hooking strategy comparison

| Approach | Stability/API coverage | Runtime overhead | Detectability | Maintenance |
|---|---|---:|---|---|
| JNI RegisterNatives replacement for selected native framework methods | Good only where the target is actually registered as a native method; not a universal hook for Java methods or Binder calls | Low after registration | Native method table / function pointer can be inspected | Medium; registration timing and class-loader details matter |
| ART method replacement/trampoline (vendored library such as LSPlant, linked into this module) | Potentially broad Java-method coverage, but depends on ART internals, hidden runtime behavior, API level, architecture, and OEM changes | Moderate during installation; low-to-moderate per invocation | Hooked entrypoints and altered method metadata may be detectable | High; requires per-ART validation, native code, compatibility matrix, and careful rollback |
| Binder/Parcel interception inside target process | Covers selected Binder transactions only if the app's client path is intercepted; transaction layouts and service versions vary | Moderate; every relevant transaction traverses proxy | Proxy/Parcel behavior can be inspected | Very high; service-specific and fragile |
| libc/PLT hooks | Useful for native libc/property calls; does not cover Java framework methods or Binder Java paths generally | Low-to-moderate | GOT/PLT and libc behavior can be inspected | High across linker namespaces, bionic, and vendor variations |
| Global system_server modification | Broad service coverage, but violates process-scoped requirement and can affect unrelated apps/device behavior | Device-wide | High-impact and visible | Unacceptable for this design |

**Decision:** do not ship a general ART trampoline in the first implementation. This is a conscious safety boundary, not an LSPosed dependency issue. A self-contained ART hook library is technically compatible with the no-external-manager requirement, but it would still create persistent per-process hooks whose ART layouts, method entrypoints, and deoptimization behavior must be validated across API 29–35 and OEM builds. Current code unloads the module library; a surviving ART hook cannot safely point into code that has been DlClose'd. Vendoring a library alone does not solve lifetime, rollback, class-loader, or crash-safety problems.

A later ART phase may be evaluated only after a dedicated API/ABI/OEM test matrix and a lifetime design (e.g. hook implementation code intentionally resident for the target process, not dangling into unloaded module memory). It must remain app-scoped.

## 4. Identifier-by-identifier feasibility

Legend: **Phase 1 candidate** means narrow implementation is plausible, not yet implemented or verified. **Deferred** means no production claim until a separately validated hook exists. **Not supported** means deliberately excluded from the safe initial scope.

| Identifier | App-visible interception point | Feasibility / decision |
|---|---|---|
| `Build.SERIAL` (deprecated static field) | Existing app-process JNI static-field write to `android.os.Build.SERIAL`, selected only when a valid identity block supplies `serial`. | **Implemented as a narrow Phase 1 field override.** App-process only; does not modify `Build.getSerial()`, `ro.serialno` in system_server, or attestation data. The field is deprecated and may be ignored by apps using the modern getter. |
| `Build.getSerial()`, `ro.serialno`, `ro.boot.serialno` | Android 12 `Build.getSerial()` calls `IDeviceIdentifiersPolicyService.getSerialForPackage()` over Binder; that service reads `ro.serialno` in `system_server`. A target app's private COW property area therefore cannot change the service result. Direct hidden property reads in the target process are a separate, non-public path and must not be described as `Build.getSerial()` support. | Deferred. Requires a separately validated app-side Binder reply hook or a system_server/global change, which remains out of scope. Permission gates and target-SDK behavior also apply. |
| IMEI per slot | `TelephonyManager.getImei(int)` / `getImei()` and legacy `getDeviceId()` | Deferred. Pure-Java framework method; ART hook required for broad coverage. Binder interception must understand telephony service transactions and permission checks; not selected. |
| MEID | `TelephonyManager.getMeid(int)` / `getMeid()`, legacy `getDeviceId()` | Deferred for the same reasons as IMEI. |
| IMSI | `TelephonyManager.getSubscriberId()` and subscription-specific methods | Deferred. Permission and carrier-privilege gates mean many apps cannot read a value; changing a Java return alone does not change service-side state. |
| ICCID | `TelephonyManager.getSimSerialNumber()` or subscription APIs | Deferred. Same ART/Binder and permission limitations. |
| Phone / line number | `TelephonyManager.getLine1Number()` and subscription APIs | Deferred. Often unavailable or empty; line-number state is carrier/SIM dependent. |
| Operator MCC/MNC and name | `TelephonyManager.getSimOperator()`, `getNetworkOperator()`, `getSimOperatorName()`, `getNetworkOperatorName()`, `SubscriptionInfo` accessors | Deferred. Must keep SIM and current-network values coherent and subscription-scoped; Java and service data paths differ. |
| ANDROID_ID / SSAID | `Settings.Secure.getString(ContentResolver, ANDROID_ID)` | Deferred. The value is scoped by Android signing key/user/device behavior and can be sourced through Settings provider/cache. A Java hook catches only the selected overload/path; ContentResolver/Binder interception is more brittle. Avoid changing Settings provider or system_server. |
| GAID | Google Play services `AdvertisingIdClient` results and its in-process Binder/client interface | Deferred. This is not an Android framework identifier; GMS versions vary, may return limited/zero IDs, and may use Binder or implementation-specific client APIs. A generic hook on one SDK method will not cover all app integrations. No GMS process/system service modification. |
| MediaDrm / Widevine `deviceUniqueId` | Native implementation registered for `android.media.MediaDrm` methods; the repository can use a JNI RegisterNatives-style replacement only after resolving the actual native registration and signature | Technically plausible but **not included in safe Phase 1** until registration names/signatures and device-specific MediaDrm behavior are verified. Returned bytes may be hardware/vendor-backed and may not be caller-controlled. Do not mutate DRM provisioning, keybox, TEE, or Widevine state. |
| Wi-Fi MAC | App-visible `WifiInfo.getMacAddress()`, `NetworkInterface.getHardwareAddress()`, and platform permission/redaction behavior | Deferred / best effort only. Multiple Java and native paths; Android commonly returns a constant/redacted address to ordinary apps. Do not change kernel/netd/interface MAC or global network state. |
| Bluetooth MAC | `BluetoothAdapter.getAddress()` and related APIs | Deferred / best effort only. Permission/privacy behavior and OEM implementation vary. Do not change controller address or Bluetooth daemon state. |
| Key-attestation identifiers | In-process construction of attestation request parameters, where an app explicitly supplies ID tags to a framework API | Limited and deferred. Only caller-provided values before Binder are conceptually process-local. The keystore service/TEE decides whether identifiers are accepted and signs hardware-backed data. Do not intercept or forge the signed certificate chain; do not alter keystored, KeyMint, TEE, or real attestation data. |
| Firebase/app-instance IDs | SDK-specific Firebase Installations / Analytics storage and service clients | Not a generic OS identifier. Deferred; app SDK versions and persisted state vary. Do not modify arbitrary app databases. |

### Android 12 serial source verification

The Android 12 AOSP `Build.getSerial()` implementation obtains `IDeviceIdentifiersPolicyService` from `ServiceManager` and calls `getSerialForPackage()`; the service checks device-identifier permissions and then reads `ro.serialno` from the service process. A per-app COW property view cannot alter the value read inside `system_server`. The deprecated `Build.SERIAL` field is separate: AOSP initializes it as a static field through `getString("no.such.thing")` and documents it as always `Build.UNKNOWN`. Device Faker now uses its existing JNI static-field writer to override only that legacy field inside the selected app process. The modern `Build.getSerial()` service response, global `ro.serialno`, and hardware-backed attestation remain unchanged.

References: [Android 12 Build.java](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android12-d1-release/core/java/android/os/Build.java), [DeviceIdentifiersPolicyService.java](https://android.googlesource.com/platform/frameworks/base/+/457bfa1180fd66bdf2e8f91897cc66743decec92/services/core/java/com/android/server/os/DeviceIdentifiersPolicyService.java), and [AOSP device identifier permission guidance](https://source.android.com/docs/core/connect/device-identifiers).

## 5. Zygisk lifecycle and code lifetime

1. `on_load`: keep existing initialization behavior; no identity hook is installed globally.
2. `pre_app_specialize`: extract package + user ID, load TOML, resolve exactly one existing merged record using current package precedence, validate the optional identity block, and install only hooks explicitly required by non-empty fields.
3. Any Java/ART hook proposal must run only after package selection and only inside this app's specialization path. It must never run in `pre_server_specialize` or zygote-global initialization.
4. Existing immediate `DlCloseModuleLibrary` is safe only for work whose installed state does not retain pointers into the module library. A trampoline whose replacement function lives in the module library cannot outlive `DlClose`. Such a hook would require a deliberately resident implementation or a separately mapped, self-contained executable image with a proven lifetime/cleanup model. This is a design exception and must be opt-in, documented, and measured.
5. Until that lifetime contract is implemented and proven, ART hooks are skipped. JNI failures must be checked; pending exceptions must be cleared before continuing to the next identifier. No panic/unwrap in hook paths.

## 6. Configuration semantics proposal

Add an optional `identity` nested record to both template and app records:

```toml
[templates.s23_ultra.identity]
mode = "static" # "static" | "seeded"
seed = "local-seed"
android_id = "7a4d9c2b3e8f11a4"
serial = "R5CW20AB12C"
# Other optional values are documented only when an implementation exists.

[[apps]]
package = "com.example.target"
[apps.identity]
mode = "static"
android_id = "7a4d9c2b3e8f11a4"
```

Compatibility requirements:
- All fields are optional; absent/empty means unchanged.
- Unknown identity keys should be ignored where the parser's existing serde policy permits.
- Do not change old top-level config behavior.
- Match current selection semantics: package key lookup checks the exact `package@userId` key before bare package fallback; within a key, direct `[[apps]]` record wins over a template; the first template in TOML order wins among multiple template claims.
- **Important:** existing `get_merged_config` merges top-level device fields; the user's requested identity policy says `[[apps]]` replaces the template as a whole record. The implementation must explicitly encode identity selection as an atomic optional block: when a direct app record is selected, its identity block replaces the template's identity block; missing identity on that selected app means no identity changes, not inheritance from the template. This avoids accidental mixed identities. Keep this behavior consistent in Rust and WebUI.
- Seeded derivation must use a cryptographic keyed derivation over (seed, normalized package key, identifier kind, slot). It must be deterministic and stable per app; do not use process RNG as part of seeded derivation.

## 7. Generator data integrity and privacy

Generator logic can be pure Rust and unit-tested on a host. However:
- Do not manufacture globally assigned or operationally valid IMEI/TAC, MEID, IMSI, ICCID issuer ranges, phone numbers, MAC OUIs, or manufacturer serials that could collide with real subscribers/devices. Luhn validity is only a checksum, not proof of valid allocation.
- TAC/OUI/carrier allocation data must have verifiable licensing/provenance and be treated as reference metadata, not random allocation authority. In particular, carrier MCC/MNC pairs can be documented as public network identifiers, but generating random subscriber MSINs/ICCID values that resemble active subscribers is not appropriate for a privacy feature.
- Safer default generator scope: cryptographically random/deterministic synthetic values in clearly reserved/test ranges where a standard exists, and non-routable/local identifiers (e.g. local unicast MAC bit pattern) with explicit test-only semantics. Where no safe reserved range exists, emit no value and ask the user to supply a value from a test SIM/lab.
- A synthetic phone number may be generated only from a formally reserved fictional/test numbering range for the selected country; otherwise leave it blank.
- `toml_edit` can preserve comments/formatting, but writes must use temp + fsync + atomic rename and create a permission-preserving timestamped backup. Do not silently overwrite a user's config.
- WebUI is static Vue in the module's WebUI. To share Rust generator code, expose a deliberate bridge (e.g. host CLI invocation or a narrow native/companion command). Do not duplicate generator logic in TypeScript and call it "shared." No network access is needed.

## 8. Risk matrix

| Mechanism / field group | Crash risk if implemented naively | Detectability | API 29–35 outlook |
|---|---|---|---|
| Existing Build static-field JNI writes | Low–medium; JNI/class init failures must be handled | Moderate | Existing path already present; exact fields and API-gates still need tests |
| Per-process serial properties | Low for property layer; Java cache/call path may disagree | Moderate | Property names vary by vendor/build; Java getter needs separate validation |
| RegisterNatives MediaDrm replacement | Medium–high without exact signature/registration timing | High for integrity-sensitive DRM clients | No claim until tested per API/vendor; native registration can move |
| ART method hook (Telephony/Settings) | High without validated ART support + rollback | High | Potentially supportable with vendored library but not safely claimable without a 29–35 matrix |
| Binder/Parcel rewrite | High; transaction mismatch can corrupt replies or break service clients | High | Service transaction layouts and object parceling change; not selected |
| GMS GAID interception | Medium–high; private GMS contract drift | High | No universal app-side entry point |
| MAC Java API hooks | Medium | Moderate–high | API redaction and OEM paths differ; no hardware/network mutation allowed |
| Attestation request parameter rewrite | Medium–high; malformed request/chain causes failure | High | Only pre-service caller fields may be altered; signed hardware data remains authentic |
| System_server / service mutation | Very high and device-wide | High | Prohibited |

## 9. Explicitly out of scope / not feasible safely

- Changing real IMEI/MEID/IMSI/ICCID or carrier/SIM state in telephony service, radio/HAL, modem, SIM, or system_server.
- Forging a hardware-backed attestation certificate, KeyMint/TEE identifiers, Widevine provisioning, DRM keybox, or cryptographic device binding.
- Promising one hook covers all GAID libraries and app-specific advertising SDKs.
- Spoofing app-specific Firebase IDs generically without modifying that app's SDK/storage.
- Spoofing Wi-Fi/Bluetooth controller addresses while also leaving hardware/network state untouched.
- Guaranteeing cross-app or cross-process identity consistency when the requested scope is process-only.
- Any hook path that requires an unverified ART offset, hard-coded per-device address, or a function pointer into code that has been DlClose'd.

## 10. Initial implementation recommendation

Phase 1 should be limited to:
1. backward-compatible identity config schema and strict field validation;
2. safe generator infrastructure and TOML editing tests, while refusing unsafe allocated-number fabrication;
3. a narrowly scoped override of the deprecated `Build.SERIAL` static field in the selected app process; do not claim support for `Build.getSerial()` or the system serial service;
4. debug-only per-identifier status logs that never include values;
5. documentation and host-side tests.

Do not enable TelephonyManager, Settings.Secure, GAID, MediaDrm, MAC, or attestation hooks in this phase. Each requires a separately reviewed native/ART implementation and real-device coverage. This is narrower than the requested identifier list, but consistent with the stated no-bootloop / process-scope / no-external-framework requirements.

## 11. Test plan and performance reporting

Before claiming support:
- host unit tests: Luhn/checksum helpers (only if retained for test-format output), parser round-trip, old-config compatibility, seeded determinism, per-package domain separation, malformed field rejection, app-vs-template atomic identity precedence, and user-suffix lookup;
- Android integration matrix: API 29, 30, 31, 32, 33, 34, 35; arm64 first; at least AOSP/Pixel plus vendor builds;
- compare configured target app to an unlisted control app, and verify that system_server and unrelated apps remain unchanged;
- verify module-only install with no LSPosed/Xposed/Riru installed;
- measure median and p95 added time in `pre_app_specialize` with identity unset vs. set; no performance number is claimed before those measurements.

Manual validation must use test apps that display values through documented public APIs. Values hidden by platform permissions/redaction should be reported as unavailable, not marked as successful spoofing.


## 12. Identity schema implementation status

The Rust configuration model now accepts optional identity blocks on both template and direct-app records. The selected block is copied atomically into MergedAppConfig: a direct app record wins as a whole, and an absent identity block on that direct record does not inherit the template block.

The schema recognizes `enabled` (optional; missing means enabled for backward compatibility), mode (static / seeded), seed, android_id, serial, and app_instance_id. When `enabled = false`, the identity block is inert and no identity field is applied. Unknown keys are ignored. Empty strings are treated as unset. Seeded mode requires a non-empty seed when a supported value is configured.

The only current runtime consumer is `identity.serial`, which is written to the deprecated `Build.SERIAL` static field inside the selected app process after the complete identity block passes validation and `identity.enabled` is not false. It does not change `Build.getSerial()` or the system serial service. `android_id`, `app_instance_id`, and seed-derived previews remain configuration/generator data and are not consumed by Android APIs. Host tests cover legacy config parsing, template selection, direct-app atomic replacement, unknown-key tolerance, and seeded-mode validation.


## 14. Identity schema v1 and synthetic generator

The configuration model recognizes `IdentityGenerate` with two explicit synthetic outputs:
- `android_id`: a 64-bit Android-ID-shaped test value, serialized as 16 lowercase hexadecimal characters by the generator.
- `app_instance_id`: a Device Faker UUIDv8 under RFC 9562.

The seed is 32 bytes, written in TOML as exactly 64 hexadecimal characters. HMAC-SHA-256 uses separate domain strings for each output and includes a length-prefixed package name plus Android user ID. The same seed/scope/output kind is deterministic; different package names and users are cryptographically domain-separated.

Static `android_id` accepts 1–16 hexadecimal characters because AOSP's `Long.toHexString` representation is not guaranteed to be zero-padded. Serial is validated as printable ASCII with a project-local 1–128 byte limit and is never generated.

The runtime treats an unresolved package scope as `None`, requests module unload for that process, and does not query a configuration key such as `@0`. A selected identity block is validated as a whole. When valid, `identity.serial` may update only `Build.SERIAL` in that app process; invalid profiles are ignored. Other identity values do not yet affect Android APIs.

**Important:** generated Android-ID-shaped and UUIDv8 values remain synthetic preview data. The legacy `Build.SERIAL` app-process field override does not change the real device serial, `Build.getSerial()`, `Settings.Secure.ANDROID_ID`, telephony state, GAID, DRM provisioning, MAC controller addresses, or hardware-backed attestation.


## 15. Shared Rust generator in the WebUI

The browser integration uses the isolated `identity-wasm` crate, which depends on the same `identity-core` package as the developer CLI. It exposes a JSON preview function only; it does not install Android hooks or write configuration files.

The WebUI obtains a 32-byte seed from the browser's cryptographic random source (`crypto.getRandomValues`) and passes that seed, the package name, and Android user ID to the Rust bridge. HMAC-SHA-256 derivation remains exclusively in `identity-core`; TypeScript does not reimplement the derivation algorithm.

For a template preview, the first assigned package is used as the preview scope. The seed remains package-independent in the template configuration; a future runtime consumer must derive separately for each actual package/user scope. A direct app profile uses that app's package/user scope.

The WebUI displays derived values as synthetic previews and stores only the seeded profile (seed + generation flags) in TOML. Static serial values remain manual input only. The UI never writes a previewed Android-ID-shaped value or UUID into a static identifier field automatically.

For a local build, install `wasm-pack` v0.15.1 and add the Rust target with `rustup target add wasm32-unknown-unknown`. From `webui/`, run `npm run build:wasm` before `npm run type-check`, `npm run dev`, or `npm run build`. The `xtask build` command now generates the WASM package automatically before running the locked npm install and Vite build. CI installs the same `wasm-pack` version, builds through `xtask`, and then type-checks the generated WebUI. Generated files under `webui/src/wasm/pkg` are build outputs, not the source of truth.

This bridge shares the generator. The Android module consumes only the configured legacy `Build.SERIAL` field at runtime; other generated identifiers remain preview/configuration data.
