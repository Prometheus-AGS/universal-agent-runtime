const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/memory-context.feature'],
  import: ['scripts/behaviors/steps/memory-context.steps.mjs'],
  parallel: 0,
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-memory-context.json`}`],
};
