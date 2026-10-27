const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/linked-kb-delete.feature'],
  import: ['scripts/behaviors/steps/linked-kb-delete.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-linked-kb-delete.json`}`],
};
