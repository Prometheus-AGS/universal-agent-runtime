const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/mcp-tool-approval.feature'],
  import: ['scripts/behaviors/steps/mcp-tool-approval.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-mcp-tool-approval.json`}`],
};
