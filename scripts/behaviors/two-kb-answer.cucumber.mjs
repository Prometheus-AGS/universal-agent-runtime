const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/two-kb-answer.feature'],
  import: ['scripts/behaviors/steps/two-kb-answer.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-two-kb-answer.json`}`],
};
