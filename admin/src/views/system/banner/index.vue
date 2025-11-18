<template>
  <GiPageLayout :margin="false" :body-style="{ padding: 0 }">
    <GiTable
      row-key="id"
      :data="dataList"
      :columns="columns"
      :loading="loading"
      :scroll="{ x: '100%', y: '100%', minWidth: 900 }"
      :pagination="pagination"
      :disabled-tools="['size']"
      @refresh="search"
    >
      <template #toolbar-left>
        <a-input-search v-model="queryForm.keyword" placeholder="搜索标题" allow-clear @search="search" />
        <a-select
          v-model="queryForm.status"
          :options="DisEnableStatusList"
          placeholder="请选择状态"
          allow-clear
          style="width: 150px"
          @change="search"
        />
        <a-button @click="reset">
          <template #icon><icon-refresh /></template>
          <template #default>重置</template>
        </a-button>
      </template>
      <template #toolbar-right>
        <a-button v-permission="['system:banner:create']" type="primary" @click="onAdd">
          <template #icon><icon-plus /></template>
          <template #default>新增</template>
        </a-button>
      </template>
      <template #imageUrl="{ record }">
        <img
          v-if="record.imageUrl"
          :src="record.imageUrl"
          alt="banner"
          style="max-width: 140px; max-height: 60px; border-radius: 4px"
        >
      </template>
      <template #status="{ record }">
        <GiCellStatus :status="record.status" />
      </template>
      <template #action="{ record }">
        <a-space>
          <a-link v-permission="['system:banner:update']" title="修改" @click="onUpdate(record)">修改</a-link>
          <a-link
            v-permission="['system:banner:delete']"
            status="danger"
            title="删除"
            @click="onDelete(record)"
          >
            删除
          </a-link>
        </a-space>
      </template>
    </GiTable>

    <BannerAddModal ref="BannerAddModalRef" @save-success="search" />
  </GiPageLayout>
</template>

<script setup lang="ts">
import type { TableInstance } from '@arco-design/web-vue'
import BannerAddModal from './BannerAddModal.vue'
import { type BannerQuery, type BannerResp, deleteBanner, listBanner } from '@/apis/system/banner'
import { DisEnableStatusList } from '@/constant/common'
import { useTable } from '@/hooks'
import { isMobile } from '@/utils'
import has from '@/utils/has'
import GiCellStatus from '@/components/GiCell/GiCellStatus.vue'

defineOptions({ name: 'SystemBanner' })

const queryForm = reactive<BannerQuery>({
  keyword: undefined,
  status: undefined,
  sort: ['id,desc'],
})

const {
  tableData: dataList,
  loading,
  pagination,
  search,
  handleDelete,
} = useTable((page) => listBanner({ ...queryForm, ...page }), { immediate: true })

const columns: TableInstance['columns'] = [
  {
    title: '序号',
    width: 66,
    align: 'center',
    render: ({ rowIndex }) => h('span', {}, rowIndex + 1 + (pagination.current - 1) * pagination.pageSize),
    fixed: !isMobile() ? 'left' : undefined,
  },
  { title: '标题', dataIndex: 'title', slotName: 'title', minWidth: 160, ellipsis: true, tooltip: true },
  { title: '图片', dataIndex: 'imageUrl', slotName: 'imageUrl', width: 180 },
  { title: '跳转链接', dataIndex: 'linkUrl', ellipsis: true, tooltip: true, minWidth: 200 },
  { title: '排序', dataIndex: 'sort', width: 80, align: 'center' },
  { title: '状态', dataIndex: 'status', slotName: 'status', width: 100, align: 'center' },
  { title: '创建时间', dataIndex: 'createTime', width: 180 },
  { title: '修改时间', dataIndex: 'updateTime', width: 180, show: false },
  {
    title: '操作',
    dataIndex: 'action',
    slotName: 'action',
    width: 160,
    align: 'center',
    fixed: !isMobile() ? 'right' : undefined,
    show: has.hasPermOr(['system:banner:update', 'system:banner:delete']),
  },
]

// 重置
const reset = () => {
  queryForm.keyword = undefined
  queryForm.status = undefined
  search()
}

const BannerAddModalRef = ref<InstanceType<typeof BannerAddModal>>()

// 新增
const onAdd = () => {
  BannerAddModalRef.value?.onAdd()
}

// 修改
const onUpdate = (record: BannerResp) => {
  BannerAddModalRef.value?.onUpdate(record.id as string)
}

// 删除
const onDelete = (record: BannerResp) => {
  return handleDelete(() => deleteBanner(record.id as string), {
    content: `是否确定删除 Banner「${record.title}」？`,
    showModal: true,
  })
}
</script>

<style scoped lang="scss"></style>

