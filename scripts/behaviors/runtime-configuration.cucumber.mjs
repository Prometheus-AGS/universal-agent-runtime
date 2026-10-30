const platform = { darwin: 'macos', linux: 'linux', win32: 'windows' }[process.platform] ?? process.platform;
const timestamp = new Date().toISOString().replaceAll(':', '-');

export default {
  paths: ['scripts/behaviors/features/runtime-configuration.feature', 'scripts/behaviors/features/anthropic-messages-parity.feature'],
  import: ['scripts/behaviors/steps/runtime-configuration.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${timestamp}-runtime-configuration-${platform}.json`}`],
};
