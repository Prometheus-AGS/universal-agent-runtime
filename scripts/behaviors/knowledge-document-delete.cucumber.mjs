const stamp = new Date().toISOString().replaceAll(':', '-');
export default {
  paths: ['scripts/behaviors/features/knowledge-document-delete.feature'],
  import: ['scripts/behaviors/steps/knowledge-document-delete.steps.mjs'],
  retry: 0,
  format: ['progress', `json:${process.env.UAR_CUCUMBER_REPORT ?? `.prometheus/verification/${stamp}-knowledge-document-delete.json`}`],
};
