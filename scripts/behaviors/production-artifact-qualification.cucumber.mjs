const timestamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/production-artifact-qualification.feature'],
  import: ['scripts/behaviors/steps/production-artifact-qualification.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${timestamp}-production-artifact-qualification.json`}`],
};
