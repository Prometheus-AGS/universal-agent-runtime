const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/native-chat-admission.feature'],
  import: ['scripts/behaviors/steps/native-chat-admission.steps.mjs'],
  retry: 0,
  parallel: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-native-chat-admission.json`}`],
};
