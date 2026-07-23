<script setup lang="ts">
import type { PortRule } from '../../types'

interface Props {
  globalIp?: string
  editPort?: PortRule | null
}

const props = withDefaults(defineProps<Props>(), {
  globalIp: '',
  editPort: null,
})

const emit = defineEmits<{
  save: [port: PortRule]
  cancel: []
}>()

const visible = defineModel<boolean>('visible', { required: true })

const isEditing = computed(() => props.editPort !== null)

const formRef = ref()
const formData = ref<{
  name: string
  local_port: number | null
  target_ip: string
  target_port: number | null
  enabled: boolean
}>({
  name: '',
  local_port: null,
  target_ip: '',
  target_port: null,
  enabled: true,
})

// 编辑模式下初始化表单数据
watch(
  () => props.editPort,
  (port) => {
    if (port) {
      formData.value = {
        name: port.name || '',
        local_port: port.local_port,
        target_ip: port.target_ip || '',
        target_port: port.target_port || null,
        enabled: port.enabled,
      }
    } else {
      formData.value = {
        name: '',
        local_port: null,
        target_ip: '',
        target_port: null,
        enabled: true,
      }
    }
  },
  { immediate: true },
)

const rules = {
  local_port: [
    {
      required: true,
      message: '请输入端口号',
      trigger: 'blur',
      validator(_rule: unknown, value: number | null) {
        if (value === null || value === undefined) return false
        return value >= 1 && value <= 65535
      },
    },
  ],
  target_ip: [
    {
      validator(_rule: unknown, value: string) {
        if (!value || value.trim() === '') return true // 可选
        // 允许 IPv4、IPv6、域名；仅禁止空白字符
        return !/\s/.test(value.trim())
      },
      message: '地址格式不正确，不能包含空格',
      trigger: 'blur',
    },
  ],
}

function handleSave() {
  formRef.value?.validate((errors: unknown) => {
    if (errors) return

    const port: PortRule = {
      local_port: formData.value.local_port!,
      name: formData.value.name.trim() || null,
      target_ip: formData.value.target_ip.trim() || null,
      target_port: formData.value.target_port || null,
      enabled: formData.value.enabled,
    }

    emit('save', port)
    visible.value = false
  })
}

function handleCancel() {
  emit('cancel')
  visible.value = false
}

// 关闭后重置表单
watch(visible, (val) => {
  if (!val) {
    formData.value = { name: '', local_port: null, target_ip: '', target_port: null, enabled: true }
  }
})
</script>

<template>
  <n-modal
    v-model:show="visible"
    preset="card"
    :title="isEditing ? '编辑端口' : '添加端口'"
    style="width: 480px"
    :mask-closable="false"
  >
    <n-form ref="formRef" :model="formData" :rules="rules" label-placement="left" label-width="90">
      <n-form-item label="名称" path="name">
        <n-input
          v-model:value="formData.name"
          placeholder="例如 MySQL-3306（可选）"
          style="width: 100%"
        />
      </n-form-item>

      <n-form-item label="监听端口" path="local_port">
        <n-input-number
          v-model:value="formData.local_port"
          :min="1"
          :max="65535"
          placeholder="例如 8080"
          :disabled="isEditing"
          style="width: 100%"
        />
      </n-form-item>

      <n-form-item label="目标地址" path="target_ip">
        <n-input
          v-model:value="formData.target_ip"
          :placeholder="`IPv4/IPv6/域名，留空则使用全局地址（${props.globalIp || '未设置'}）`"
        />
      </n-form-item>

      <n-form-item label="目标端口" path="target_port">
        <n-input-number
          v-model:value="formData.target_port"
          :min="1"
          :max="65535"
          placeholder="留空则与监听端口相同"
          style="width: 100%"
        />
      </n-form-item>

      <n-form-item label="启用状态">
        <n-switch v-model:value="formData.enabled" />
      </n-form-item>
    </n-form>

    <template #footer>
      <n-space justify="end">
        <n-button @click="handleCancel">取消</n-button>
        <n-button type="primary" @click="handleSave">
          {{ isEditing ? '保存' : '添加' }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>
