const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/knowledge-regeneration.feature'],
  import: ['scripts/behaviors/steps/knowledge-regeneration.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-knowledge-regeneration.json`}`],
};
