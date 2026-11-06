<template>
  <div class="identity-lab">
    <section class="hero glass-effect">
      <div class="hero-copy">
        <p class="eyebrow">{{ t('identityLab.eyebrow') }}</p>
        <h2>{{ t('identityLab.title') }}</h2>
        <p>{{ t('identityLab.subtitle') }}</p>
      </div>
      <span class="hero-mark"><Fingerprint :size="30" /></span>
    </section>

    <section class="panel glass-effect">
      <div class="panel-heading">
        <div>
          <h3>{{ t('identityLab.appTargetTitle') }}</h3>
          <p>{{ t('identityLab.appTargetDesc') }}</p>
        </div>
        <span class="state-pill">{{ t('identityLab.localScope') }}</span>
      </div>
      <el-select
        v-model="selectedPackage"
        filterable
        clearable
        :placeholder="t('identityLab.selectPackage')"
        class="package-select"
        :loading="appsStore.loading"
      >
        <el-option
          v-for="app in appsStore.installedApps"
          :key="app.packageName"
          :label="`${app.appName} · ${app.packageName}`"
          :value="app.packageName"
        />
      </el-select>
      <p v-if="!selectedPackage" class="hint">{{ t('identityLab.choosePackageHint') }}</p>
    </section>

    <template v-if="selectedPackage">
      <section class="panel glass-effect">
        <div class="panel-heading">
          <div>
            <h3>{{ t('identityLab.appIdentityTitle') }}</h3>
            <p>{{ t('identityLab.appIdentityDesc') }}</p>
            <p class="scope-line">{{ profileScopeLabel }}</p>
          </div>
          <span class="state-pill" :class="profileEnabled ? 'state-pill--green' : ''">
            {{ profileEnabled ? t('identityLab.profileActive') : t('identityLab.profileDisabled') }}
          </span>
        </div>

        <div class="profile-enable-row">
          <div>
            <strong>{{ t('identityLab.enableTitle') }}</strong>
            <small>{{ t('identityLab.enableDesc') }}</small>
          </div>
          <el-switch v-model="profileEnabled" />
        </div>

        <el-form label-position="top">
          <el-form-item :label="t('identityLab.modeTitle')">
            <el-radio-group v-model="mode">
              <el-radio-button value="static">{{ t('identityLab.staticMode') }}</el-radio-button>
              <el-radio-button value="seeded">{{ t('identityLab.seededMode') }}</el-radio-button>
            </el-radio-group>
          </el-form-item>

          <template v-if="mode === 'static'">
            <el-form-item :label="t('identityLab.androidIdStatic')">
              <el-input v-model="androidId" maxlength="16" :placeholder="t('identityLab.androidIdPlaceholder')" />
              <div class="field-actions">
                <el-button size="small" @click="generateStaticValue('android_id')">{{ t('identityLab.generateTestValue') }}</el-button>
                <el-button v-if="androidId" size="small" text @click="copyValue(androidId)">{{ t('identityLab.copy') }}</el-button>
              </div>
            </el-form-item>
            <el-form-item :label="t('identityLab.appInstanceStatic')">
              <el-input v-model="appInstanceId" :placeholder="t('identityLab.uuidPlaceholder')" />
              <div class="field-actions">
                <el-button size="small" @click="generateStaticValue('app_instance_id')">{{ t('identityLab.generateTestValue') }}</el-button>
                <el-button v-if="appInstanceId" size="small" text @click="copyValue(appInstanceId)">{{ t('identityLab.copy') }}</el-button>
              </div>
            </el-form-item>
          </template>

          <template v-else>
            <div class="field-card">
              <div class="field-top">
                <div>
                  <strong>{{ t('identityLab.generateAndroidId') }}</strong>
                  <small>{{ t('identityLab.androidIdPreviewNote') }}</small>
                </div>
                <el-switch v-model="generateAndroidId" />
              </div>
              <div v-if="preview && generateAndroidId" class="value-row">
                <code>{{ preview.identifiers.android_id }}</code>
                <el-button text @click="copyValue(preview.identifiers.android_id)">{{ t('identityLab.copy') }}</el-button>
              </div>
              <el-input
                v-if="!generateAndroidId"
                v-model="androidId"
                maxlength="16"
                :placeholder="t('identityLab.manualAndroidIdPlaceholder')"
              />
            </div>

            <div class="field-card">
              <div class="field-top">
                <div>
                  <strong>{{ t('identityLab.generateAppInstance') }}</strong>
                  <small>{{ t('identityLab.appInstanceNote') }}</small>
                </div>
                <el-switch v-model="generateAppInstanceId" />
              </div>
              <div v-if="preview && generateAppInstanceId" class="value-row">
                <code>{{ preview.identifiers.app_instance_id }}</code>
                <el-button text @click="copyValue(preview.identifiers.app_instance_id)">{{ t('identityLab.copy') }}</el-button>
              </div>
              <el-input
                v-if="!generateAppInstanceId"
                v-model="appInstanceId"
                :placeholder="t('identityLab.manualUuidPlaceholder')"
              />
            </div>

            <div class="actions">
              <el-button type="primary" :loading="busy" @click="generatePreview">
                <WandSparkles :size="16" /> {{ t('identityLab.generateSeedPreview') }}
              </el-button>
              <el-button :disabled="!seed || busy" @click="refreshPreview">{{ t('identityLab.refreshPreview') }}</el-button>
            </div>

            <div v-if="seed" class="seed-box">
              <label>{{ t('identityLab.privateSeedLabel') }}</label>
              <div class="value-row">
                <code>{{ seed }}</code>
                <el-button text @click="copyValue(seed)">{{ t('identityLab.copy') }}</el-button>
              </div>
              <p>{{ t('identityLab.deterministicSeedNote') }}</p>
            </div>
          </template>

          <div class="field-card">
            <div class="field-top">
              <div>
                <strong>{{ t('identityLab.serialTitle') }}</strong>
                <small>{{ t('identityLab.serialNote') }}</small>
              </div>
              <el-switch v-model="serialEnabled" />
            </div>
            <el-input
              v-model="serial"
              :disabled="!serialEnabled"
              maxlength="128"
              :placeholder="t('identityLab.serialPlaceholder')"
            />
            <div class="field-actions">
              <el-button size="small" :disabled="!serialEnabled" @click="generateTestSerial">{{ t('identityLab.generateTestSerial') }}</el-button>
              <el-button v-if="serial" size="small" text @click="copyValue(serial)">{{ t('identityLab.copy') }}</el-button>
            </div>
            <span class="runtime-pill">{{ t('identityLab.limitedSupport') }}</span>
          </div>
        </el-form>

        <p v-if="error" class="error">{{ error }}</p>
        <p class="notice">{{ runtimeNotice }}</p>
      </section>

      <section class="panel glass-effect">
        <div class="panel-heading">
          <div>
            <h3>{{ t('identityLab.protectedTitle') }}</h3>
            <p>{{ t('identityLab.protectedDesc') }}</p>
          </div>
          <span class="state-pill">{{ t('identityLab.notChanged') }}</span>
        </div>
        <div v-for="item in protectedIdentifiers" :key="item.nameKey" class="protected-row">
          <div>
            <strong>{{ t(item.nameKey) }}</strong>
            <small>{{ t(item.descriptionKey) }}</small>
          </div>
          <span class="unavailable-pill">{{ t('identityLab.notImplemented') }}</span>
        </div>
        <p class="notice">{{ t('identityLab.protectedNotice') }}</p>
      </section>

      <div class="save-bar">
        <el-button @click="loadSelectedProfile">{{ t('identityLab.discard') }}</el-button>
        <el-button type="primary" :loading="saving" @click="saveProfile">{{ t('identityLab.save') }}</el-button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { Fingerprint, WandSparkles } from '@lucide/vue'
import { ElMessage } from 'element-plus'
import { useI18n } from '../utils/i18n'
import { useAppsStore } from '../stores/apps'
import { useConfigStore } from '../stores/config'
import {
  deriveSyntheticIdentityPreview,
  createIdentitySeedHex,
  isIdentityConfigValid,
  type SyntheticIdentityPreview,
} from '../utils/identity'
import type { IdentityConfig } from '../types'
import { configWriteKey, parsePackageUser } from '../utils/package'

const { t } = useI18n()
const appsStore = useAppsStore()
const configStore = useConfigStore()
const selectedPackage = ref('')
const profileEnabled = ref(true)
const mode = ref<'static' | 'seeded'>('static')
const seed = ref('')
const androidId = ref('')
const appInstanceId = ref('')
const serial = ref('')
const serialEnabled = ref(false)
const generateAndroidId = ref(true)
const generateAppInstanceId = ref(true)
const preview = ref<SyntheticIdentityPreview | null>(null)
const busy = ref(false)
const saving = ref(false)
const error = ref('')
let previewRequestId = 0

const profileScopeLabel = computed(() => {
  if (!selectedPackage.value) return ''
  const scope = parsePackageUser(selectedPackage.value)
  return t('identityLab.scope', { package: scope.base, userId: scope.userId })
})

const runtimeNotice = computed(() => t('identityLab.runtimeNotice'))
const protectedIdentifiers = [
  { nameKey: 'identityLab.protected.gsfName', descriptionKey: 'identityLab.protected.gsfDesc' },
  { nameKey: 'identityLab.protected.gaidName', descriptionKey: 'identityLab.protected.gaidDesc' },
  { nameKey: 'identityLab.protected.appSetName', descriptionKey: 'identityLab.protected.appSetDesc' },
  { nameKey: 'identityLab.protected.imeiName', descriptionKey: 'identityLab.protected.imeiDesc' },
  { nameKey: 'identityLab.protected.imsiName', descriptionKey: 'identityLab.protected.imsiDesc' },
  { nameKey: 'identityLab.protected.iccidName', descriptionKey: 'identityLab.protected.iccidDesc' },
  { nameKey: 'identityLab.protected.simSubscriptionName', descriptionKey: 'identityLab.protected.simSubscriptionDesc' },
  { nameKey: 'identityLab.protected.mobileName', descriptionKey: 'identityLab.protected.mobileDesc' },
  { nameKey: 'identityLab.protected.operatorName', descriptionKey: 'identityLab.protected.operatorDesc' },
  { nameKey: 'identityLab.protected.eidName', descriptionKey: 'identityLab.protected.eidDesc' },
  { nameKey: 'identityLab.protected.drmName', descriptionKey: 'identityLab.protected.drmDesc' },
  { nameKey: 'identityLab.protected.wifiMacName', descriptionKey: 'identityLab.protected.wifiMacDesc' },
  { nameKey: 'identityLab.protected.bssidName', descriptionKey: 'identityLab.protected.bssidDesc' },
  { nameKey: 'identityLab.protected.bluetoothName', descriptionKey: 'identityLab.protected.bluetoothDesc' },
  { nameKey: 'identityLab.protected.hardwareName', descriptionKey: 'identityLab.protected.hardwareDesc' },
]

watch(selectedPackage, () => loadSelectedProfile())

onMounted(async () => {
  await configStore.bootstrap()
  await appsStore.ensureUserAppsLoaded()
  if (!selectedPackage.value && appsStore.installedApps.length) {
    selectedPackage.value = appsStore.installedApps[0].packageName
  }
})

function loadSelectedProfile() {
  previewRequestId += 1
  error.value = ''
  preview.value = null
  if (!selectedPackage.value) {
    profileEnabled.value = true
    mode.value = 'static'
    seed.value = ''
    androidId.value = ''
    appInstanceId.value = ''
    serial.value = ''
    serialEnabled.value = false
    generateAndroidId.value = false
    generateAppInstanceId.value = false
    return
  }

  const resolved = configStore.getPackageConfig(selectedPackage.value)
  const identity = resolved?.identity
  profileEnabled.value = identity?.enabled !== false
  mode.value = identity?.mode === 'seeded' ? 'seeded' : 'static'
  seed.value = identity?.seed ?? ''
  androidId.value = identity?.android_id ?? ''
  appInstanceId.value = identity?.app_instance_id ?? ''
  serial.value = identity?.serial ?? ''
  serialEnabled.value = Boolean(identity?.serial?.trim())
  generateAndroidId.value = identity?.generate?.android_id === true
  generateAppInstanceId.value = identity?.generate?.app_instance_id === true

  if (mode.value === 'seeded' && seed.value && (generateAndroidId.value || generateAppInstanceId.value)) {
    void refreshPreview()
  }
}

async function refreshPreview() {
  const packageName = selectedPackage.value
  const seedHex = seed.value
  if (!packageName || !seedHex) {
    preview.value = null
    return
  }

  const requestId = ++previewRequestId
  try {
    const scope = parsePackageUser(packageName)
    const result = await deriveSyntheticIdentityPreview(scope.base, scope.userId, seedHex)
    if (requestId !== previewRequestId || packageName !== selectedPackage.value || seedHex !== seed.value) {
      return
    }
    preview.value = result
    error.value = ''
  } catch (cause) {
    if (
      requestId !== previewRequestId ||
      packageName !== selectedPackage.value ||
      seedHex !== seed.value
    ) return
    preview.value = null
    error.value = cause instanceof Error ? cause.message : String(cause)
  }
}

async function generatePreview() {
  if (!selectedPackage.value || busy.value) return
  busy.value = true
  error.value = ''
  try {
    mode.value = 'seeded'
    generateAndroidId.value = true
    generateAppInstanceId.value = true
    androidId.value = ''
    appInstanceId.value = ''
    seed.value = createIdentitySeedHex()
    await refreshPreview()
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    busy.value = false
  }
}

async function generateStaticValue(kind: 'android_id' | 'app_instance_id') {
  if (!selectedPackage.value || busy.value) return
  busy.value = true
  error.value = ''
  try {
    const packageName = selectedPackage.value
    const scope = parsePackageUser(packageName)
    const testSeed = createIdentitySeedHex()
    const values = await deriveSyntheticIdentityPreview(scope.base, scope.userId, testSeed)
    if (selectedPackage.value !== packageName) return
    if (kind === 'android_id') androidId.value = values.identifiers.android_id
    else appInstanceId.value = values.identifiers.app_instance_id
    ElMessage.success(t('identityLab.generated'))
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    busy.value = false
  }
}

function generateTestSerial() {
  try {
    serial.value = `DF-TEST-${createIdentitySeedHex().slice(0, 12).toUpperCase()}`
    serialEnabled.value = true
    ElMessage.success(t('identityLab.serialGenerated'))
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  }
}

async function copyValue(value: string) {
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success(t('identityLab.copied'))
  } catch {
    error.value = t('identityLab.errorClipboard')
  }
}

async function saveProfile() {
  if (!selectedPackage.value || saving.value) return
  saving.value = true
  error.value = ''
  try {
    const identity: IdentityConfig = { enabled: profileEnabled.value, mode: mode.value }
    const cleanSerial = serialEnabled.value ? serial.value.trim() : ''
    if (serialEnabled.value && !cleanSerial) {
      throw new Error(t('identityLab.errorSerialRequired'))
    }
    if (cleanSerial) identity.serial = cleanSerial

    if (mode.value === 'static') {
      if (androidId.value.trim()) identity.android_id = androidId.value.trim()
      if (appInstanceId.value.trim()) identity.app_instance_id = appInstanceId.value.trim()
    } else {
      const generate: NonNullable<IdentityConfig['generate']> = {}
      if (generateAndroidId.value) generate.android_id = true
      else if (androidId.value.trim()) identity.android_id = androidId.value.trim()
      if (generateAppInstanceId.value) generate.app_instance_id = true
      else if (appInstanceId.value.trim()) identity.app_instance_id = appInstanceId.value.trim()

      if (generate.android_id || generate.app_instance_id) {
        identity.mode = 'seeded'
        identity.seed = seed.value.trim()
        identity.generate = generate
      } else {
        identity.mode = 'static'
      }
    }

    if (!isIdentityConfigValid(identity)) {
      throw new Error(t('identityLab.errorInvalidProfile'))
    }

    const resolvedMatches = configStore.resolvePackageMatchesFor(selectedPackage.value)
    const writeKey = configWriteKey(selectedPackage.value, resolvedMatches?.key)
    const existingApp = configStore.getApps().find((app) => app.package === writeKey)
    configStore.setApp({
      ...(existingApp ?? {}),
      package: writeKey,
      identity,
    })
    await configStore.saveConfig()

    if (identity.mode === 'seeded') await refreshPreview()
    else preview.value = null
    ElMessage.success(profileEnabled.value ? t('identityLab.savedActive') : t('identityLab.savedDisabled'))
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.identity-lab{display:flex;flex-direction:column;gap:1rem;padding-bottom:1.5rem;color:var(--text)}
.hero,.panel{border:1px solid var(--border);border-radius:1rem;background:var(--card-bg);box-shadow:0 4px 16px var(--shadow);padding:1rem}
.hero{display:flex;justify-content:space-between;align-items:center;gap:1rem;background:linear-gradient(135deg,var(--card-bg),var(--background))}
.eyebrow{font-size:.68rem;letter-spacing:.12em;color:var(--primary);font-weight:700;margin:0 0 .35rem}
.hero h2{font-size:1.45rem;margin:0 0 .4rem}
.hero p,.panel-heading p{color:var(--text-secondary);font-size:.82rem;line-height:1.45;margin:0}
.hero-mark{display:grid;place-items:center;width:3.25rem;height:3.25rem;border-radius:1rem;background:var(--background);color:var(--primary);flex-shrink:0}
.panel-heading,.field-top{display:flex;align-items:flex-start;justify-content:space-between;gap:.8rem}
.panel-heading{margin-bottom:1rem}
.panel-heading h3{font-size:1rem;margin:0 0 .25rem}
.state-pill,.runtime-pill,.unavailable-pill{font-size:.62rem;font-weight:700;letter-spacing:.04em;border-radius:99px;padding:.35rem .55rem;background:var(--background);color:var(--text-secondary);white-space:nowrap}
.state-pill--green{color:#21845b}
.package-select{width:100%}
.profile-enable-row{display:flex;align-items:center;justify-content:space-between;gap:.8rem;padding:.9rem 0 1.1rem;margin-bottom:.4rem;border-bottom:1px solid var(--border)}
.profile-enable-row strong{display:block;font-size:.9rem}
.profile-enable-row small{display:block;color:var(--text-secondary);font-size:.73rem;line-height:1.45;margin-top:.2rem}
.scope-line{font-size:.74rem!important;margin-top:.3rem!important;color:var(--text-secondary)}
.field-actions{display:flex;flex-wrap:wrap;gap:.35rem;margin-top:.35rem}
.hint,.notice,.seed-box p{font-size:.76rem;line-height:1.5;color:var(--text-secondary)}
.field-card{padding:.9rem;border:1px solid var(--border);border-radius:.8rem;margin:.7rem 0;background:var(--background)}
.field-top{align-items:center;margin-bottom:.75rem}
.field-top strong,.protected-row strong{display:block;font-size:.88rem}
.field-top small,.protected-row small{display:block;font-size:.72rem;color:var(--text-secondary);line-height:1.4;margin-top:.25rem}
.field-card code,.seed-box code{display:block;overflow-wrap:anywhere;font:.78rem/1.5 ui-monospace,monospace;padding:.55rem;border-radius:.5rem;background:var(--card-bg);margin-top:.55rem}
.value-row{display:flex;align-items:center;gap:.35rem;min-width:0}
.value-row code{flex:1;min-width:0;overflow-wrap:anywhere;font:.76rem/1.5 ui-monospace,monospace;padding:.5rem;border-radius:.5rem;background:var(--card-bg)}
.runtime-pill{display:inline-block;margin-top:.6rem;color:#21845b}
.actions{display:flex;flex-wrap:wrap;gap:.55rem;margin-top:1rem}
.seed-box{margin-top:1rem;padding:.8rem;border:1px dashed var(--border);border-radius:.75rem}
.seed-box label{font-size:.72rem;font-weight:700}
.protected-row{display:flex;justify-content:space-between;align-items:center;gap:.75rem;padding:.8rem 0;border-bottom:1px solid var(--border)}
.protected-row:last-of-type{border-bottom:0}
.unavailable-pill{color:var(--text-secondary)}
.notice{padding:.7rem;border-radius:.65rem;background:var(--background);margin:.85rem 0 0}
.error{font-size:.8rem;color:var(--danger,#d44);margin:.75rem 0}
.save-bar{display:flex;justify-content:flex-end;gap:.6rem;position:sticky;bottom:.5rem;padding:.7rem;background:var(--card-bg);border:1px solid var(--border);border-radius:.9rem;box-shadow:0 4px 18px var(--shadow)}
@media(max-width:420px){.field-top{align-items:flex-start}.protected-row{align-items:flex-start;flex-direction:column}.save-bar{justify-content:stretch}.save-bar :deep(.el-button){flex:1}}
</style>
