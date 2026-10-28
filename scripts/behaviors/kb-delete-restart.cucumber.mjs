const platform = { darwin: 'macos', linux: 'linux', win32: 'windows' }[process.platform] ?? process.platform;
const timestamp = new Date().toISOString().replaceAll(':', '-');

export default {
  paths: ['scripts/behaviors/features/kb-delete-restart.feature'],
  import: ['scripts/behaviors/steps/kb-delete-restart.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${timestamp}-kb-delete-restart-${platform}.json`}`],
};
