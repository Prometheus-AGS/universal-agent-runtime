const platform = { darwin: 'macos', linux: 'linux', win32: 'windows' }[process.platform] ?? process.platform;
const timestamp = new Date().toISOString().replaceAll(':', '-');

export default {
  paths: ['scripts/behaviors/features/skill-catalog-activation.feature'],
  import: ['scripts/behaviors/steps/skill-catalog-activation.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${timestamp}-skill-catalog-activation-${platform}.json`}`],
};
