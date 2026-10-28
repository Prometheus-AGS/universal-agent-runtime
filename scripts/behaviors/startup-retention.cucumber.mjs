const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/startup-retention.feature'],
  import: ['scripts/behaviors/steps/startup-retention.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-startup-retention.json`}`],
};
