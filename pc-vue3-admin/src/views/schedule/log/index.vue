<template>
  <GiPageLayout>
    <div class="schedule-disabled">
      <p>任务调度日志功能已移除，当前系统不再记录或展示定时任务执行情况。</p>
    </div>
  </GiPageLayout>
</template>

<script setup lang="ts">
defineOptions({ name: 'ScheduleLog' })
</script>

<style scoped lang="scss">
.schedule-disabled {
  padding: 24px;
}
</style>
