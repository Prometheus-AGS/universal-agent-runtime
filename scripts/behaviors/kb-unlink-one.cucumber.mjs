const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/kb-unlink-one.feature'],
  import: ['scripts/behaviors/steps/kb-unlink-one.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-kb-unlink-one.json`}`],
};
