import type { Preview } from '@storybook/react-vite'
import '../src/index.css'

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
       color: /(background|color)$/i,
       date: /Date$/i,
      },
    },

    a11y: {
      // 'error' -- this repo's a11y-architect convention is fail-closed,
      // not advisory (see a11y-architect agent, WCAG 2.2 compliance).
      test: 'error'
    }
  },
};

export default preview;
