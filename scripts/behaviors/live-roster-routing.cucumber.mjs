const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/live-roster-routing.feature'],
  import: ['scripts/behaviors/steps/live-roster-routing.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-live-roster-routing.json`}`],
};
