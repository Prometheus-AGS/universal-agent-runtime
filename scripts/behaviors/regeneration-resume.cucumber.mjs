const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/regeneration-resume.feature'],
  import: ['scripts/behaviors/steps/regeneration-resume.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-regeneration-resume.json`}`],
};
