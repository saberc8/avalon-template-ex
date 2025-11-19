<template>
  <GiPageLayout>
    <div class="schedule-disabled">
      <p>任务调度模块已移除，当前系统不再支持前端定时任务管理。</p>
    </div>
  </GiPageLayout>
</template>

<script setup lang="ts">
defineOptions({ name: 'ScheduleJob' })
</script>

<style scoped lang="scss">
.schedule-disabled {
  padding: 24px;
}
</style>
