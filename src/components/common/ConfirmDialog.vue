<script setup lang="ts">
interface Props {
  title?: string
  content: string
  positiveText?: string
  negativeText?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '确认操作',
  positiveText: '确认',
  negativeText: '取消',
})

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const visible = defineModel<boolean>('visible', { required: true })

function handlePositive() {
  emit('confirm')
  visible.value = false
}

function handleNegative() {
  emit('cancel')
  visible.value = false
}
</script>

<template>
  <n-modal v-model:show="visible" preset="dialog" :title="props.title">
    <div>{{ props.content }}</div>
    <template #action>
      <n-space justify="end">
        <n-button @click="handleNegative">{{ props.negativeText }}</n-button>
        <n-button type="error" @click="handlePositive">
          {{ props.positiveText }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>
