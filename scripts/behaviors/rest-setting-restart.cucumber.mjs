const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/rest-setting-restart.feature'],
  import: ['scripts/behaviors/steps/rest-setting-restart.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-rest-setting-restart.json`}`],
};
