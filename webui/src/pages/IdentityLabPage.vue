<template>
  <div class="identity-lab">
    <section class="hero glass-effect">
      <div class="hero-copy">
        <p class="eyebrow">ZYGISK · APP-SCOPED PROFILE</p>
        <h2>Identity Lab</h2>
        <p>Central de identidade por aplicativo. Configure valores sintéticos e veja claramente o que o runtime realmente aplica.</p>
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
        @change="loadSelectedProfile"
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
            <h3>Identificadores sintéticos</h3>
            <p>Valores de teste com escopo por pacote e usuário Android.</p>
          </div>
          <span class="state-pill state-pill--green">GERAÇÃO LOCAL</span>
        </div>

        <div class="field-card">
          <div class="field-top">
            <div>
              <strong>Android ID em formato sintético</strong>
              <small>Prévia gerada por seed; ainda não aplicada a Settings.Secure.ANDROID_ID.</small>
            </div>
            <el-switch v-model="generateAndroidId" />
          </div>
          <code v-if="preview">{{ preview.identifiers.android_id }}</code>
          <span v-else class="empty-value">Gere uma seed para visualizar</span>
        </div>

        <div class="field-card">
          <div class="field-top">
            <div>
              <strong>App Instance UUID</strong>
              <small>UUIDv8 sintético para testes; não substitui IDs internos de SDKs de terceiros.</small>
            </div>
            <el-switch v-model="generateAppInstanceId" />
          </div>
          <code v-if="preview">{{ preview.identifiers.app_instance_id }}</code>
          <span v-else class="empty-value">Gere uma seed para visualizar</span>
        </div>

        <div class="field-card">
          <div class="field-top">
            <div>
              <strong>Legacy Build.SERIAL</strong>
              <small>Único campo desta seção consumido atualmente pelo hook Zygisk; somente o campo estático legado no processo selecionado.</small>
            </div>
            <el-switch v-model="serialEnabled" />
          </div>
          <el-input
            v-model="serial"
            :disabled="!serialEnabled"
            maxlength="128"
            placeholder="Digite um valor de teste"
          />
          <span class="runtime-pill">APLICAÇÃO LIMITADA IMPLEMENTADA</span>
        </div>

        <div class="actions">
          <el-button type="primary" :loading="busy" :disabled="!selectedPackage" @click="generatePreview">
            <WandSparkles :size="16" /> Gerar seed e prévia
          </el-button>
          <el-button :disabled="!seed" @click="generatePreview">Atualizar prévia</el-button>
        </div>
        <div v-if="seed" class="seed-box">
          <label>Seed privada · 32 bytes</label>
          <code>{{ seed }}</code>
          <p>Guarde esta seed em local seguro. Ela determina os valores sintéticos para este pacote e usuário.</p>
        </div>
        <p v-if="error" class="error">{{ error }}</p>
        <p class="notice">{{ runtimeNotice }}</p>
      </section>

      <section class="panel glass-effect">
        <div class="panel-heading">
          <div>
            <h3>Identificadores de hardware e serviços</h3>
            <p>Visibilidade honesta sobre disponibilidade no Android.</p>
          </div>
          <span class="state-pill">NÃO ALTERADOS</span>
        </div>
        <div v-for="item in protectedIdentifiers" :key="item.name" class="protected-row">
          <div>
            <strong>{{ item.name }}</strong>
            <small>{{ item.description }}</small>
          </div>
          <span class="unavailable-pill">Não suportado</span>
        </div>
        <p class="notice">IMEI, MEID, IMSI, ICCID, número telefônico, endereços físicos de rede, MediaDrm/Widevine e atestação dependem de serviços, hardware ou provisionamento protegido. Esta interface não simula alteração real desses identificadores.</p>
      </section>

      <div class="save-bar">
        <el-button @click="loadSelectedProfile">Descartar alterações</el-button>
        <el-button type="primary" :loading="saving" @click="saveProfile">Salvar perfil</el-button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { Fingerprint, WandSparkles } from '@lucide/vue'
import { useAppsStore } from '../stores/apps'
import { useConfigStore } from '../stores/config'
import { deriveSyntheticIdentityPreview, createIdentitySeedHex, isIdentityConfigValid, type SyntheticIdentityPreview } from '../utils/identity'
import type { IdentityConfig } from '../types'
import { parsePackageUser } from '../utils/package'

const appsStore = useAppsStore()
const configStore = useConfigStore()
const selectedPackage = ref('')
const seed = ref('')
const serial = ref('')
const serialEnabled = ref(false)
const generateAndroidId = ref(true)
const generateAppInstanceId = ref(true)
const preview = ref<SyntheticIdentityPreview | null>(null)
const busy = ref(false)
const saving = ref(false)
const error = ref('')

const runtimeNotice = 'O runtime atual aplica somente Legacy Build.SERIAL. Android ID e App Instance UUID são gerados e armazenados como prévias/configuração, mas ainda não são interceptados nas APIs Android.'
const protectedIdentifiers = [
  { name: 'IMEI / MEID / IMSI / ICCID', description: 'Identificadores de modem, SIM e assinante; sem alteração de identidade de telecomunicações.' },
  { name: 'Wi-Fi MAC / BSSID / SSID', description: 'Endereços e dados de rede reais não são modificados.' },
  { name: 'Bluetooth MAC', description: 'Endereço do controlador Bluetooth permanece intacto.' },
  { name: 'MediaDrm / Widevine ID', description: 'Provisionamento DRM e chaves protegidas permanecem intactos.' },
  { name: 'Hardware ID / atestação', description: 'Identidade de hardware, TEE, KeyMint e atestação permanecem intactos.' },
  { name: 'Número móvel / operador SIM', description: 'Estado de assinante e operadora não é falsificado.' },
]

const currentIdentity = computed<IdentityConfig | undefined>(() => {
  if (!selectedPackage.value) return undefined
  return configStore.getPackageConfig(selectedPackage.value)?.identity
})

watch(selectedPackage, () => loadSelectedProfile())

onMounted(async () => {
  await appsStore.ensureUserAppsLoaded()
  if (!selectedPackage.value && appsStore.installedApps.length) {
    selectedPackage.value = appsStore.installedApps[0].packageName
    loadSelectedProfile()
  }
})

function loadSelectedProfile() {
  error.value = ''
  preview.value = null
  const identity = currentIdentity.value
  seed.value = identity?.seed ?? ''
  serial.value = identity?.serial ?? ''
  serialEnabled.value = Boolean(identity?.serial)
  generateAndroidId.value = identity?.generate?.android_id !== false
  generateAppInstanceId.value = identity?.generate?.app_instance_id !== false
  if (seed.value && (generateAndroidId.value || generateAppInstanceId.value)) {
    void refreshPreview()
  }
}

async function refreshPreview() {
  if (!selectedPackage.value || !seed.value) return
  try {
    const scope = parsePackageUser(selectedPackage.value)
    preview.value = await deriveSyntheticIdentityPreview(scope.base, scope.userId, seed.value)
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  }
}

async function generatePreview() {
  if (!selectedPackage.value || busy.value) return
  busy.value = true
  error.value = ''
  try {
    if (!seed.value) seed.value = createIdentitySeedHex()
    await refreshPreview()
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    busy.value = false
  }
}

async function saveProfile() {
  if (!selectedPackage.value || saving.value) return
  saving.value = true
  error.value = ''
  try {
    const identity: IdentityConfig = {
      mode: 'seeded',
      seed: seed.value,
      serial: serialEnabled.value ? serial.value.trim() : undefined,
      generate: {
        android_id: generateAndroidId.value,
        app_instance_id: generateAppInstanceId.value,
      },
    }
    if (!generateAndroidId.value && !generateAppInstanceId.value) {
      identity.mode = 'static'
      delete identity.seed
      delete identity.generate
    }
    if (serialEnabled.value && !serial.value.trim()) {
      throw new Error('Informe o valor de Serial ou desative esse campo.')
    }
    if (!isIdentityConfigValid(identity)) {
      throw new Error('Configuração inválida. Verifique seed, Serial e opções de geração.')
    }
    const current = configStore.getPackageConfig(selectedPackage.value)
    configStore.setApp({
      ...(current ?? {}),
      package: selectedPackage.value,
      identity,
    })
    await configStore.saveConfig()
    if (identity.seed) await refreshPreview()
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
.field-card code,.seed-box code{display:block;overflow-wrap:anywhere;font: .78rem/1.5 ui-monospace,monospace;padding:.55rem;border-radius:.5rem;background:var(--card-bg);margin-top:.55rem}
.empty-value{font-size:.75rem;color:var(--text-secondary)}
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
