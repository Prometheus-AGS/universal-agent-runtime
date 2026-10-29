const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/native-tool-approval.feature'],
  import: ['scripts/behaviors/steps/native-tool-approval.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-native-tool-approval.json`}`],
};
