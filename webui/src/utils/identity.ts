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
let wasmInitialized = false

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
  if (!wasmInitialized) {
    await wasm.default()
    wasmInitialized = true
  }
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
    preview.synthetic !== true
  ) {
    throw new Error('The Rust identity generator returned an unexpected response.')
  }

  return preview as SyntheticIdentityPreview
}
