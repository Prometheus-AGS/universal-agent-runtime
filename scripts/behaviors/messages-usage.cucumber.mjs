const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/anthropic-messages-parity.feature'],
  tags: '@messages-usage',
  import: ['scripts/behaviors/steps/messages-usage.steps.mjs'],
  retry: 0,
  parallel: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-messages-usage.json`}`],
};
