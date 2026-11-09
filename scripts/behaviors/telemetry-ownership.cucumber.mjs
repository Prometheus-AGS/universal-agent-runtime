const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/telemetry-ownership.feature'],
  import: ['scripts/behaviors/steps/telemetry-ownership.steps.mjs'],
  retry: 0,
  parallel: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-telemetry-ownership.json`}`],
};
