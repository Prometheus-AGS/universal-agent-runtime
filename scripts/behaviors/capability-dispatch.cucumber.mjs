const platform = { darwin: 'macos', linux: 'linux', win32: 'windows' }[process.platform] ?? process.platform;
const timestamp = new Date().toISOString().replaceAll(':', '-');

export default {
  paths: ['scripts/behaviors/features/capability-dispatch.feature'],
  import: ['scripts/behaviors/steps/capability-dispatch.steps.mjs'],
  retry: 0,
  parallel: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${timestamp}-capability-dispatch-${platform}.json`}`],
};
