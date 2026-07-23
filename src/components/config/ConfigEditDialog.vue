<script setup lang="ts">
interface Props {
  editConfig?: { name: string; global_ip: string } | null
}

const props = withDefaults(defineProps<Props>(), {
  editConfig: null,
})

const emit = defineEmits<{
  save: [data: { name: string; global_ip: string }]
  cancel: []
}>()

const visible = defineModel<boolean>('visible', { required: true })

const isEditing = computed(() => props.editConfig !== null)

const formRef = ref()
const formData = ref({
  name: '',
  global_ip: '',
})

watch(
  () => props.editConfig,
  (cfg) => {
    if (cfg) {
      formData.value = { name: cfg.name, global_ip: cfg.global_ip }
    } else {
      formData.value = { name: '', global_ip: '' }
    }
  },
  { immediate: true },
)

const rules = {
  name: [{ required: true, message: '请输入配置名称', trigger: 'blur' }],
  global_ip: [
    { required: true, message: '请输入全局 IP', trigger: 'blur' },
    {
      validator(_rule: unknown, value: string) {
        return /^(\d{1,3}\.){3}\d{1,3}$/.test(value.trim())
      },
      message: 'IP 地址格式不正确',
      trigger: 'blur',
    },
  ],
}

function handleSave() {
  formRef.value?.validate((errors: unknown) => {
    if (errors) return
    emit('save', {
      name: formData.value.name.trim(),
      global_ip: formData.value.global_ip.trim(),
    })
    visible.value = false
  })
}
</script>

<template>
  <n-modal
    v-model:show="visible"
    preset="card"
    :title="isEditing ? '编辑配置' : '新建配置'"
    style="width: 440px"
    :mask-closable="false"
  >
    <n-form
      ref="formRef"
      :model="formData"
      :rules="rules"
      label-placement="left"
      label-width="90"
    >
      <n-form-item label="配置名称" path="name">
        <n-input
          v-model:value="formData.name"
          placeholder="例如 工作环境"
          :disabled="isEditing"
        />
      </n-form-item>
      <n-form-item label="全局 IP" path="global_ip">
        <n-input v-model:value="formData.global_ip" placeholder="例如 192.168.1.100" />
      </n-form-item>
    </n-form>
    <template #footer>
      <n-space justify="end">
        <n-button @click="visible = false">取消</n-button>
        <n-button type="primary" @click="handleSave">{{
          isEditing ? '保存' : '创建'
        }}</n-button>
      </n-space>
    </template>
  </n-modal>
</template>
