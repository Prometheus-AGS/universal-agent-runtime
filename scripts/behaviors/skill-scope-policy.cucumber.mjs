const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/skill-scope-policy.feature'],
  import: ['scripts/behaviors/steps/skill-scope-policy.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-skill-scope-policy.json`}`],
};
