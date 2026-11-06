const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/embedding-usage.feature'],
  import: ['scripts/behaviors/steps/embedding-usage.steps.mjs'],
  retry: 0,
  parallel: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-embedding-usage.json`}`],
};
