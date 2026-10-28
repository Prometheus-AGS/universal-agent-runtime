const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/embedding-same-dimension.feature'],
  import: ['scripts/behaviors/steps/embedding-same-dimension.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-embedding-same-dimension.json`}`],
};
