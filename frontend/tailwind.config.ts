import type { Config } from "tailwindcss";
import tailwindcssAnimate from "tailwindcss-animate";

// KnowMe token bindings (uar-ui-token-convergence). Token VALUES live in
// src/index.css; this file only maps them to utilities. `--border` and
// `--sidebar-border` are the literal `transparent` (Flat 2.0), so they bind
// as raw var() instead of hsl() channels.
const config: Config = {
	darkMode: ["class"],
	content: ["./index.html", "./src/**/*.{ts,tsx}", "./packages/a2ui-uar/src/**/*.{ts,tsx}"],
	theme: {
		extend: {
			colors: {
				border: 'var(--border)',
				input: 'hsl(var(--input))',
				ring: 'hsl(var(--ring))',
				background: 'hsl(var(--background))',
				chrome: 'hsl(var(--chrome))',
				surface: 'hsl(var(--surface))',
				foreground: 'hsl(var(--foreground))',
				'fg-sub': 'hsl(var(--fg-sub))',
				'fg-faint': 'hsl(var(--fg-faint))',
				primary: {
					DEFAULT: 'hsl(var(--primary))',
					foreground: 'hsl(var(--primary-foreground))'
				},
				secondary: {
					DEFAULT: 'hsl(var(--secondary))',
					foreground: 'hsl(var(--secondary-foreground))'
				},
				destructive: {
					DEFAULT: 'hsl(var(--destructive))',
					foreground: 'hsl(var(--destructive-foreground))'
				},
				muted: {
					DEFAULT: 'hsl(var(--muted))',
					foreground: 'hsl(var(--muted-foreground))'
				},
				accent: {
					DEFAULT: 'hsl(var(--accent))',
					foreground: 'hsl(var(--accent-foreground))'
				},
				popover: {
					DEFAULT: 'hsl(var(--popover))',
					foreground: 'hsl(var(--popover-foreground))'
				},
				card: {
					DEFAULT: 'hsl(var(--card))',
					foreground: 'hsl(var(--card-foreground))',
					hov: 'hsl(var(--card-hov))'
				},
				ember: {
					DEFAULT: 'hsl(var(--ember))',
					2: 'hsl(var(--ember-2))',
					soft: 'hsl(var(--ember-soft))'
				},
				cyan: 'hsl(var(--cyan))',
				success: 'hsl(var(--success))',
				warning: 'hsl(var(--warning))',
				sidebar: {
					DEFAULT: 'hsl(var(--sidebar-background))',
					foreground: 'hsl(var(--sidebar-foreground))',
					primary: 'hsl(var(--sidebar-primary))',
					'primary-foreground': 'hsl(var(--sidebar-primary-foreground))',
					accent: 'hsl(var(--sidebar-accent))',
					'accent-foreground': 'hsl(var(--sidebar-accent-foreground))',
					border: 'var(--sidebar-border)',
					ring: 'hsl(var(--sidebar-ring))'
				}
			},
			borderRadius: {
				sm: 'calc(var(--radius) * 0.6)',
				md: 'calc(var(--radius) * 0.8)',
				lg: 'var(--radius)',
				xl: 'calc(var(--radius) * 1.4)',
				'2xl': 'calc(var(--radius) * 1.8)',
				'3xl': 'calc(var(--radius) * 2.2)',
				'4xl': 'calc(var(--radius) * 2.6)'
			},
			fontFamily: {
				sans: ['Geist Variable', 'Inter', 'system-ui', 'sans-serif'],
				mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
				display: ['Space Grotesk', 'Inter', 'sans-serif'],
				body: ['Geist Variable', 'Inter', 'system-ui', 'sans-serif'],
				ui: ['Geist Variable', 'Inter', 'system-ui', 'sans-serif']
			},
			keyframes: {
				'accordion-down': {
					from: { height: '0' },
					to: { height: 'var(--radix-accordion-content-height)' }
				},
				'accordion-up': {
					from: { height: 'var(--radix-accordion-content-height)' },
					to: { height: '0' }
				},
				'fade-in': {
					from: { opacity: '0' },
					to: { opacity: '1' }
				},
				'slide-in-from-bottom': {
					from: { transform: 'translateY(8px)', opacity: '0' },
					to: { transform: 'translateY(0)', opacity: '1' }
				},
				shimmer: {
					'0%': { transform: 'translateX(-200%)' },
					'100%': { transform: 'translateX(300%)' }
				}
			},
			animation: {
				'accordion-down': 'accordion-down 0.2s ease-out',
				'accordion-up': 'accordion-up 0.2s ease-out',
				'fade-in': 'fade-in 0.15s ease-out',
				in: 'fade-in 0.15s ease-out',
				shimmer: 'shimmer 1.4s ease-in-out infinite'
			}
		}
	},
	plugins: [tailwindcssAnimate],
};

export default config;
