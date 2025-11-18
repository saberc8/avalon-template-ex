<template>
  <a-modal
    v-model:visible="visible"
    :title="title"
    :mask-closable="false"
    :esc-to-close="false"
    :width="width >= 520 ? 520 : '100%'"
    draggable
    @before-ok="save"
    @close="reset"
  >
    <GiForm ref="formRef" v-model="form" :columns="columns" layout="vertical" />
  </a-modal>
</template>

<script setup lang="tsx">
import { Message } from '@arco-design/web-vue'
import { useWindowSize } from '@vueuse/core'
import { addBanner, getBanner, updateBanner } from '@/apis/system/banner'
import { type ColumnItem, GiForm } from '@/components/GiForm'
import { useResetReactive } from '@/hooks'

const emit = defineEmits<{
  (e: 'save-success'): void
}>()

const { width } = useWindowSize()

const dataId = ref('')
const visible = ref(false)
const isUpdate = computed(() => !!dataId.value)
const title = computed(() => (isUpdate.value ? '修改 Banner' : '新增 Banner'))
const formRef = ref<InstanceType<typeof GiForm>>()

// 表单数据
const [form, resetForm] = useResetReactive({
  title: '',
  imageUrl: '',
  linkUrl: '',
  sort: 1,
  status: 1,
  remark: '',
})

// 表单列配置
const columns: ColumnItem[] = reactive([
  {
    label: '标题',
    field: 'title',
    type: 'input',
    span: 24,
    props: {
      maxLength: 100,
      placeholder: '请输入 Banner 标题',
      allowClear: true,
    },
    rules: [{ required: true, message: '请输入标题' }],
  },
  {
    label: '图片地址',
    field: 'imageUrl',
    type: 'input',
    span: 24,
    props: {
      maxLength: 255,
      placeholder: '请输入图片地址（支持外链或文件服务地址）',
      allowClear: true,
    },
    rules: [{ required: true, message: '请输入图片地址' }],
  },
  {
    label: '跳转链接',
    field: 'linkUrl',
    type: 'input',
    span: 24,
    props: {
      maxLength: 255,
      placeholder: '请输入点击后跳转链接（可选）',
      allowClear: true,
    },
  },
  {
    label: '排序',
    field: 'sort',
    type: 'input-number',
    span: 24,
    props: {
      min: 1,
      mode: 'button',
    },
  },
  {
    label: '状态',
    field: 'status',
    type: 'switch',
    span: 24,
    props: {
      type: 'round',
      checkedValue: 1,
      uncheckedValue: 2,
      checkedText: '启用',
      uncheckedText: '禁用',
    },
  },
  {
    label: '备注',
    field: 'remark',
    type: 'textarea',
    span: 24,
    props: {
      maxLength: 255,
      placeholder: '请输入备注信息（可选）',
      autoSize: { minRows: 2, maxRows: 4 },
      allowClear: true,
    },
  },
])

// 重置表单
const reset = () => {
  formRef.value?.formRef?.resetFields()
  resetForm()
}

// 保存
const save = async () => {
  try {
    const isInvalid = await formRef.value?.formRef?.validate()
    if (isInvalid) return false
    if (isUpdate.value) {
      await updateBanner(form, dataId.value)
      Message.success('修改成功')
    } else {
      await addBanner(form)
      Message.success('新增成功')
    }
    emit('save-success')
    return true
  } catch (error) {
    return false
  }
}

// 新增
const onAdd = () => {
  reset()
  dataId.value = ''
  visible.value = true
}

// 修改
const onUpdate = async (id: string) => {
  reset()
  dataId.value = id
  const { data } = await getBanner(id)
  Object.assign(form, data)
  visible.value = true
}

defineExpose({ onAdd, onUpdate })
</script>

<style scoped lang="scss"></style>

