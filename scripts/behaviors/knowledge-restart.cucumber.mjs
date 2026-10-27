const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/knowledge-restart.feature'],
  import: ['scripts/behaviors/steps/knowledge-restart.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-knowledge-restart.json`}`],
};
