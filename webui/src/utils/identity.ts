import type { IdentityConfig } from '../types'

export interface SyntheticIdentityPreview {
  synthetic: true
  package: string
  android_user_id: number
  seed_hex: string
  identifiers: {
    android_id: string
    app_instance_id: string
  }
  notice: string
}

type IdentityWasmModule = typeof import('../wasm/pkg/identity_wasm')

let wasmModulePromise: Promise<IdentityWasmModule> | null = null
let wasmInitPromise: Promise<void> | null = null

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

/** Obtain seed material from the browser's cryptographic RNG, never Math.random(). */
export function createIdentitySeedHex(): string {
  const cryptoProvider = globalThis.crypto
  if (!cryptoProvider?.getRandomValues) {
    throw new Error('A cryptographically secure random source is unavailable in this WebView.')
  }

  const seed = new Uint8Array(32)
  cryptoProvider.getRandomValues(seed)
  return bytesToHex(seed)
}

async function loadIdentityWasm(): Promise<IdentityWasmModule> {
  if (!wasmModulePromise) {
    wasmModulePromise = import('../wasm/pkg/identity_wasm').catch((error: unknown) => {
      wasmModulePromise = null
      throw error
    })
  }

  const wasm = await wasmModulePromise
  if (!wasmInitPromise) {
    wasmInitPromise = wasm.default().catch((error: unknown) => {
      wasmInitPromise = null
      throw error
    })
  }
  await wasmInitPromise
  return wasm
}

/**
 * Derive preview values in Rust. TypeScript only obtains the random seed and
 * presents the result; it does not duplicate the identity derivation algorithm.
 */
export async function deriveSyntheticIdentityPreview(
  packageName: string,
  androidUserId: number,
  seedHex: string,
): Promise<SyntheticIdentityPreview> {
  const wasm = await loadIdentityWasm()
  const json = wasm.derive_identity_json(packageName, androidUserId, seedHex)
  const preview: unknown = JSON.parse(json)

  if (
    typeof preview !== 'object' ||
    preview === null ||
    !('synthetic' in preview) ||
    preview.synthetic !== true ||
    !('package' in preview) ||
    typeof preview.package !== 'string' ||
    !('android_user_id' in preview) ||
    typeof preview.android_user_id !== 'number' ||
    !('seed_hex' in preview) ||
    typeof preview.seed_hex !== 'string' ||
    !('identifiers' in preview) ||
    typeof preview.identifiers !== 'object' ||
    preview.identifiers === null ||
    !('android_id' in preview.identifiers) ||
    typeof preview.identifiers.android_id !== 'string' ||
    !('app_instance_id' in preview.identifiers) ||
    typeof preview.identifiers.app_instance_id !== 'string' ||
    !('notice' in preview) ||
    typeof preview.notice !== 'string'
  ) {
    throw new Error('The Rust identity generator returned an unexpected response.')
  }

  return preview as SyntheticIdentityPreview
}


/** Mirror the Rust IdentityConfig::validate rules before the WebUI persists a profile. */
export function isIdentityConfigValid(profile: IdentityConfig | undefined): boolean {
  if (!profile) return true

  const mode = profile.mode ?? 'static'
  if (mode !== 'static' && mode !== 'seeded') return false

  const hasAndroidId = Boolean(profile.android_id?.trim())
  const hasSerial = Boolean(profile.serial?.trim())
  const hasAppInstanceId = Boolean(profile.app_instance_id?.trim())
  const generateAndroidId = profile.generate?.android_id === true
  const generateAppInstanceId = profile.generate?.app_instance_id === true
  const generateAny = generateAndroidId || generateAppInstanceId
  const configured =
    hasAndroidId || hasSerial || hasAppInstanceId || generateAny

  if (
    hasAndroidId &&
    !/^[0-9a-f]{1,16}$/i.test(profile.android_id?.trim() ?? '')
  ) {
    return false
  }

  if (
    hasSerial &&
    !/^[\x20-\x7e]{1,128}$/.test(profile.serial?.trim() ?? '')
  ) {
    return false
  }

  if (
    hasAppInstanceId &&
    !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(
      profile.app_instance_id?.trim() ?? '',
    )
  ) {
    return false
  }

  if ((generateAndroidId && hasAndroidId) || (generateAppInstanceId && hasAppInstanceId)) {
    return false
  }

  if (generateAny && mode !== 'seeded') return false
  if (mode === 'seeded' && configured && !generateAny) return false

  const hasSeed = Boolean(profile.seed?.trim())
  if (generateAny && !/^[0-9a-f]{64}$/i.test(profile.seed?.trim() ?? '')) {
    return false
  }
  if (!generateAny && hasSeed) return false

  return true
}
