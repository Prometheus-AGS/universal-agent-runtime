export default {
  paths: ['scripts/behaviors/features/media-effects.feature'],
  import: ['scripts/behaviors/steps/media-effects.steps.mjs'],
  parallel: 0,
  retry: 0,
  format: ['progress', ...(process.env.UAR_CUCUMBER_REPORT ? ['json:' + process.env.UAR_CUCUMBER_REPORT] : [])],
};
