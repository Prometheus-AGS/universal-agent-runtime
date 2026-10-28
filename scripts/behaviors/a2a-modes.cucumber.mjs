const platform = { darwin: 'macos', linux: 'linux', win32: 'windows' }[process.platform] ?? process.platform;
const timestamp = new Date().toISOString().replaceAll(':', '-');

export default {
  paths: ['scripts/behaviors/features/a2a-modes.feature'],
  import: ['scripts/behaviors/steps/a2a-modes.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${timestamp}-a2a-modes-${platform}.json`}`],
};
