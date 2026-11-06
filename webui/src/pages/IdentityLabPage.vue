<template>
  <div class="identity-lab">
    <section class="hero glass-effect">
      <div class="hero-copy">
        <p class="eyebrow">ZYGISK · PERFIL POR APLICATIVO</p>
        <h2>Laboratório de Identidade</h2>
        <p>Configure perfis locais de teste e confira exatamente quais valores o runtime atual consegue aplicar.</p>
      </div>
      <span class="hero-mark"><Fingerprint :size="30" /></span>
    </section>

    <section class="panel glass-effect">
      <div class="panel-heading">
        <div>
          <h3>Aplicativo de destino</h3>
          <p>O perfil é salvo para um pacote específico.</p>
        </div>
        <span class="state-pill">ESCOPO LOCAL</span>
      </div>
      <el-select
        v-model="selectedPackage"
        filterable
        clearable
        placeholder="Selecione ou pesquise um aplicativo"
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
      <p v-if="!selectedPackage" class="hint">Escolha um aplicativo para carregar, editar e salvar seu perfil.</p>
    </section>

    <template v-if="selectedPackage">
      <section class="panel glass-effect">
        <div class="panel-heading">
          <div>
            <h3>Identidade do aplicativo</h3>
            <p>Escolha valores manuais ou gere identificadores sintéticos determinísticos.</p>
          </div>
          <span class="state-pill state-pill--green">POR PACOTE / USUÁRIO</span>
        </div>

        <el-form label-position="top">
          <el-form-item label="Modo do perfil">
            <el-radio-group v-model="mode">
              <el-radio-button value="static">Valores manuais</el-radio-button>
              <el-radio-button value="seeded">Derivado por seed</el-radio-button>
            </el-radio-group>
          </el-form-item>

          <template v-if="mode === 'static'">
            <el-form-item label="Android ID sintético (hexadecimal)">
              <el-input v-model="androidId" maxlength="16" placeholder="Ex.: 7a4d9c2b3e8f11a4" />
            </el-form-item>
            <el-form-item label="App Instance UUID sintético">
              <el-input v-model="appInstanceId" placeholder="UUID versão 8" />
            </el-form-item>
          </template>

          <template v-else>
            <div class="field-card">
              <div class="field-top">
                <div>
                  <strong>Gerar Android ID sintético</strong>
                  <small>Prévia em formato Android ID; não altera Settings.Secure.ANDROID_ID.</small>
                </div>
                <el-switch v-model="generateAndroidId" />
              </div>
              <div v-if="preview && generateAndroidId" class="value-row">
                <code>{{ preview.identifiers.android_id }}</code>
                <el-button text @click="copyValue(preview.identifiers.android_id)">Copiar</el-button>
              </div>
              <el-input
                v-if="!generateAndroidId"
                v-model="androidId"
                maxlength="16"
                placeholder="Valor hexadecimal manual"
              />
            </div>

            <div class="field-card">
              <div class="field-top">
                <div>
                  <strong>Gerar App Instance UUID</strong>
                  <small>UUIDv8 sintético de teste; não substitui IDs internos de SDKs de terceiros.</small>
                </div>
                <el-switch v-model="generateAppInstanceId" />
              </div>
              <div v-if="preview && generateAppInstanceId" class="value-row">
                <code>{{ preview.identifiers.app_instance_id }}</code>
                <el-button text @click="copyValue(preview.identifiers.app_instance_id)">Copiar</el-button>
              </div>
              <el-input
                v-if="!generateAppInstanceId"
                v-model="appInstanceId"
                placeholder="UUID manual"
              />
            </div>

            <div class="actions">
              <el-button type="primary" :loading="busy" @click="generatePreview">
                <WandSparkles :size="16" /> Gerar seed e prévia
              </el-button>
              <el-button :disabled="!seed || busy" @click="refreshPreview">Atualizar prévia</el-button>
            </div>

            <div v-if="seed" class="seed-box">
              <label>Seed privada · 32 bytes / 64 caracteres hexadecimais</label>
              <div class="value-row">
                <code>{{ seed }}</code>
                <el-button text @click="copyValue(seed)">Copiar</el-button>
              </div>
              <p>O mesmo pacote, usuário e seed produzem a mesma prévia. Guarde a seed em local seguro.</p>
            </div>
          </template>

          <div class="field-card">
            <div class="field-top">
              <div>
                <strong>Legacy Build.SERIAL</strong>
                <small>O hook atual altera somente o campo estático legado no processo selecionado. Build.getSerial() permanece inalterado.</small>
              </div>
              <el-switch v-model="serialEnabled" />
            </div>
            <el-input
              v-model="serial"
              :disabled="!serialEnabled"
              maxlength="128"
              placeholder="Digite um valor de teste (ASCII)"
            />
            <span class="runtime-pill">SUPORTE LIMITADO IMPLEMENTADO</span>
          </div>
        </el-form>

        <p v-if="error" class="error">{{ error }}</p>
        <p class="notice">{{ runtimeNotice }}</p>
      </section>

      <section class="panel glass-effect">
        <div class="panel-heading">
          <div>
            <h3>Identificadores de hardware e serviços</h3>
            <p>Estado real de suporte: sem interruptores fictícios.</p>
          </div>
          <span class="state-pill">NÃO ALTERADOS</span>
        </div>
        <div v-for="item in protectedIdentifiers" :key="item.name" class="protected-row">
          <div>
            <strong>{{ item.name }}</strong>
            <small>{{ item.description }}</small>
          </div>
          <span class="unavailable-pill">Não implementado</span>
        </div>
        <p class="notice">IMEI, MEID, IMSI, ICCID, número telefônico, endereços físicos de rede, MediaDrm/Widevine e atestação dependem de serviços, hardware ou provisionamento protegido. Esta página não simula que esses identificadores reais foram alterados.</p>
      </section>

      <div class="save-bar">
        <el-button @click="loadSelectedProfile">Descartar alterações</el-button>
        <el-button type="primary" :loading="saving" @click="saveProfile">Salvar perfil</el-button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { Fingerprint, WandSparkles } from '@lucide/vue'
import { ElMessage } from 'element-plus'
import { useAppsStore } from '../stores/apps'
import { useConfigStore } from '../stores/config'
import {
  deriveSyntheticIdentityPreview,
  createIdentitySeedHex,
  isIdentityConfigValid,
  type SyntheticIdentityPreview,
} from '../utils/identity'
import type { IdentityConfig } from '../types'
import { parsePackageUser } from '../utils/package'

const appsStore = useAppsStore()
const configStore = useConfigStore()
const selectedPackage = ref('')
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

const runtimeNotice = 'No runtime atual, somente Legacy Build.SERIAL é consumido por um hook. Android ID e App Instance UUID são armazenados/visualizados como dados sintéticos, mas ainda não são interceptados nas APIs Android.'
const protectedIdentifiers = [
  { name: 'IMEI / MEID / IMSI / ICCID / SIM Serial', description: 'Identificadores de modem, SIM e assinante; não há alteração de identidade de telecomunicações.' },
  { name: 'SIM Subscription IDs / número móvel / operador', description: 'Estado da assinatura, número e operadora não são modificados.' },
  { name: 'Wi-Fi MAC / BSSID / SSID', description: 'Endereços físicos e dados de rede reais permanecem intactos.' },
  { name: 'Bluetooth MAC', description: 'O endereço do controlador Bluetooth permanece intacto.' },
  { name: 'MediaDrm / Widevine ID', description: 'Provisionamento DRM, TEE, KeyMint e chaves protegidas permanecem intactos.' },
  { name: 'Hardware ID / atestação', description: 'Identidade de hardware e atestação respaldada por hardware permanecem intactas.' },
]

watch(selectedPackage, () => loadSelectedProfile())

onMounted(async () => {
  await appsStore.ensureUserAppsLoaded()
  if (!selectedPackage.value && appsStore.installedApps.length) {
    selectedPackage.value = appsStore.installedApps[0].packageName
  }
})

function loadSelectedProfile() {
  error.value = ''
  preview.value = null
  if (!selectedPackage.value) return

  const existing = configStore.getApps().find((app) => app.package === selectedPackage.value)
  const identity = existing?.identity
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
  if (!selectedPackage.value || !seed.value) return
  try {
    const scope = parsePackageUser(selectedPackage.value)
    preview.value = await deriveSyntheticIdentityPreview(scope.base, scope.userId, seed.value)
    error.value = ''
  } catch (cause) {
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

async function copyValue(value: string) {
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success('Copiado')
  } catch {
    error.value = 'Não foi possível copiar. Verifique a permissão da área de transferência do WebView.'
  }
}

async function saveProfile() {
  if (!selectedPackage.value || saving.value) return
  saving.value = true
  error.value = ''
  try {
    const identity: IdentityConfig = { mode: mode.value }
    const cleanSerial = serialEnabled.value ? serial.value.trim() : ''
    if (serialEnabled.value && !cleanSerial) {
      throw new Error('Informe o valor do Serial ou desative esse campo.')
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
      throw new Error('Configuração inválida. Confira os formatos dos valores e a seed hexadecimal de 64 caracteres.')
    }

    const existingApp = configStore.getApps().find((app) => app.package === selectedPackage.value)
    configStore.setApp({
      ...(existingApp ?? {}),
      package: selectedPackage.value,
      identity,
    })
    await configStore.saveConfig()

    if (identity.mode === 'seeded') await refreshPreview()
    else preview.value = null
    ElMessage.success('Perfil salvo')
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
