const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/memory-lifecycle.feature'],
  import: ['scripts/behaviors/steps/memory-lifecycle.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-memory-lifecycle.json`}`],
};
