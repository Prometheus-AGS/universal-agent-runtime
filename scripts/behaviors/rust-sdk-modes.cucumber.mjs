const platform = { darwin: 'macos', linux: 'linux', win32: 'windows' }[process.platform] ?? process.platform;
const timestamp = new Date().toISOString().replaceAll(':', '-');

export default {
  paths: ['scripts/behaviors/features/rust-sdk-modes.feature'],
  import: ['scripts/behaviors/steps/rust-sdk-modes.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${timestamp}-rust-sdk-modes-${platform}.json`}`],
};
