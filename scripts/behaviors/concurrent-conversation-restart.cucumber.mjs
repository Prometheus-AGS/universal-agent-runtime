const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/concurrent-conversation-restart.feature'],
  import: ['scripts/behaviors/steps/concurrent-conversation-restart.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-concurrent-conversation-restart.json`}`],
};
