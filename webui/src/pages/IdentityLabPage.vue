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
.identity-lab{--identity-radius:.85rem;display:flex;flex-direction:column;gap:.7rem;padding:0 0 1rem;color:var(--text)}
.hero,.panel{min-width:0;border:1px solid var(--border);border-radius:var(--identity-radius);background:var(--card-bg);box-shadow:none;padding:.85rem}
.hero{display:flex;align-items:center;justify-content:space-between;gap:.8rem}
.eyebrow{margin:0 0 .25rem;color:var(--primary);font-size:.64rem;font-weight:700;letter-spacing:.1em}
.hero h2{margin:0 0 .25rem;font-size:1.2rem;line-height:1.25}
.hero p,.panel-heading p{margin:0;color:var(--text-secondary);font-size:.78rem;line-height:1.45}
.hero-mark{display:grid;place-items:center;flex:0 0 2.7rem;width:2.7rem;height:2.7rem;border:1px solid var(--border);border-radius:.8rem;background:var(--background);color:var(--primary)}
.panel-heading,.field-top{display:flex;align-items:flex-start;justify-content:space-between;gap:.65rem}
.panel-heading{margin-bottom:.75rem}
.panel-heading h3{margin:0 0 .2rem;font-size:.94rem;line-height:1.3}
.state-pill,.runtime-pill,.unavailable-pill{display:inline-flex;align-items:center;border:1px solid var(--border);border-radius:99px;padding:.25rem .48rem;background:var(--background);color:var(--text-secondary);font-size:.6rem;font-weight:650;letter-spacing:.02em;white-space:nowrap}
.state-pill--green{color:#159568;border-color:rgba(21,149,104,.25)}
.package-select{width:100%}
.profile-enable-row{display:flex;align-items:center;justify-content:space-between;gap:.7rem;margin-bottom:.3rem;padding:.65rem 0 .8rem;border-bottom:1px solid var(--border)}
.profile-enable-row strong{display:block;font-size:.84rem}
.profile-enable-row small{display:block;margin-top:.2rem;color:var(--text-secondary);font-size:.7rem;line-height:1.4}
.scope-line{margin-top:.25rem!important;font-size:.7rem!important;color:var(--text-secondary)}
.field-actions{display:flex;flex-wrap:wrap;gap:.3rem;margin-top:.3rem}
.hint,.notice,.seed-box p{color:var(--text-secondary);font-size:.72rem;line-height:1.45}
.field-card{margin:.55rem 0;padding:.7rem;border:1px solid var(--border);border-radius:.72rem;background:var(--background)}
.field-top{align-items:center;margin-bottom:.55rem}
.field-top strong,.protected-row strong{display:block;font-size:.82rem}
.field-top small,.protected-row small{display:block;margin-top:.2rem;color:var(--text-secondary);font-size:.68rem;line-height:1.4}
.field-card code,.seed-box code{display:block;overflow-wrap:anywhere;margin-top:.45rem;padding:.45rem;border-radius:.45rem;background:var(--card-bg);font: .72rem/1.45 ui-monospace,SFMono-Regular,monospace}
.value-row{display:flex;align-items:center;gap:.3rem;min-width:0}
.value-row code{flex:1;min-width:0;overflow-wrap:anywhere;padding:.42rem;border-radius:.45rem;background:var(--card-bg);font:.7rem/1.45 ui-monospace,SFMono-Regular,monospace}
.runtime-pill{margin-top:.45rem;color:#159568}
.actions{display:flex;flex-wrap:wrap;gap:.45rem;margin-top:.8rem}
.seed-box{margin-top:.75rem;padding:.7rem;border:1px dashed var(--border);border-radius:.7rem}
.seed-box label{font-size:.68rem;font-weight:700}
.protected-row{display:flex;align-items:center;justify-content:space-between;gap:.65rem;padding:.62rem 0;border-bottom:1px solid var(--border)}
.protected-row:last-of-type{border-bottom:0}
.unavailable-pill{flex-shrink:0}
.notice{margin:.7rem 0 0;padding:.6rem .65rem;border:1px solid var(--border);border-radius:.6rem;background:var(--background)}
.error{margin:.65rem 0;color:var(--danger,#d44);font-size:.76rem}
.save-bar{position:sticky;bottom:.35rem;z-index:3;display:flex;justify-content:flex-end;gap:.45rem;padding:.55rem;background:var(--card-bg);border:1px solid var(--border);border-radius:.8rem;box-shadow:0 5px 18px var(--shadow)}
@media(max-width:420px){.panel{padding:.72rem}.hero{padding:.75rem}.hero h2{font-size:1.08rem}.field-top{align-items:flex-start}.protected-row{align-items:flex-start}.save-bar{justify-content:stretch}.save-bar :deep(.el-button){flex:1}}
</style>
