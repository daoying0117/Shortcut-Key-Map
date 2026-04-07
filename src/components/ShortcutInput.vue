<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  normalizeCombo,
  normalizeFromKeyboardEvent,
  selectableModifiers,
  selectablePrimaryKeys,
  splitCombo,
} from '../utils/hotkey'
import { t } from '../i18n'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const mode = ref<'record' | 'select'>('record')
const selectedModifiers = ref<string[]>([])
const selectedPrimaryKey = ref('')
const isRecording = ref(false)

const recordPlaceholder = computed(() => {
  if (isRecording.value) {
    return t('recordingPlaceholder')
  }
  return t('idlePlaceholder')
})

watch(
  () => props.modelValue,
  (combo) => {
    const { modifiers, key } = splitCombo(combo)
    selectedModifiers.value = [...modifiers]
    selectedPrimaryKey.value = key
  },
  { immediate: true },
)

watch([selectedModifiers, selectedPrimaryKey], () => {
  if (mode.value !== 'select') {
    return
  }
  const normalized = normalizeCombo(selectedModifiers.value, selectedPrimaryKey.value)
  emit('update:modelValue', normalized)
})

function handleKeydown(event: KeyboardEvent) {
  event.preventDefault()
  const normalized = normalizeFromKeyboardEvent(event)
  if (!normalized) {
    return
  }
  emit('update:modelValue', normalized)
}

function clearValue() {
  emit('update:modelValue', '')
}
</script>

<template>
  <div class="shortcut-input">
    <div class="mode-switch">
      <button
        type="button"
        class="ghost-btn"
        :class="{ active: mode === 'record' }"
        @click="mode = 'record'"
      >
        {{ t('inputModeRecord') }}
      </button>
      <button
        type="button"
        class="ghost-btn"
        :class="{ active: mode === 'select' }"
        @click="mode = 'select'"
      >
        {{ t('inputModeSelect') }}
      </button>
      <button type="button" class="ghost-btn danger" @click="clearValue">{{ t('clear') }}</button>
    </div>

    <div v-if="mode === 'record'" class="record-panel">
      <input
        class="record-input"
        :value="modelValue"
        :placeholder="recordPlaceholder"
        readonly
        @focus="isRecording = true"
        @blur="isRecording = false"
        @keydown="handleKeydown"
      />
    </div>

    <div v-else class="select-panel">
      <div class="modifier-row">
        <label v-for="modifier in selectableModifiers" :key="modifier" class="checkbox-pill">
          <input v-model="selectedModifiers" type="checkbox" :value="modifier" />
          <span>{{ modifier }}</span>
        </label>
      </div>

      <select v-model="selectedPrimaryKey" class="key-select">
        <option value="">{{ t('selectPrimaryKey') }}</option>
        <option v-for="key in selectablePrimaryKeys" :key="key" :value="key">
          {{ key }}
        </option>
      </select>
    </div>
  </div>
</template>
