import type { Preview } from '@storybook/react-vite'
import '@fontsource-variable/inter'
import '@fontsource-variable/space-grotesk'
import '@fontsource-variable/jetbrains-mono'
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
