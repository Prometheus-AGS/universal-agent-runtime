const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/runtime-measurement.feature'],
  import: ['scripts/behaviors/steps/runtime-measurement.steps.mjs'],
  retry: 0,
  parallel: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-runtime-measurement.json`}`],
};
