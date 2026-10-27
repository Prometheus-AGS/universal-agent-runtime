const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/skill-reattachment.feature'],
  import: ['scripts/behaviors/steps/skill-reattachment.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-skill-reattachment.json`}`],
};
