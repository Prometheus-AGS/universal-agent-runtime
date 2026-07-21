// KnowMe brand monogram — ported from desktop/src/shared/components/KnowMeLogo.tsx
// (uar-ui-token-convergence). Token references adapted to UAR's hsl-channel vars.
interface KnowMeLogoProps {
  size?: number;
  className?: string;
}

export function KnowMeLogo({ size = 28, className }: KnowMeLogoProps) {
  return (
    <svg viewBox="0 0 200 200" width={size} height={size} className={className} aria-hidden="true">
      <g transform="translate(50, 38)">
        <rect x="0" y="0" width="20" height="124" rx="10" fill="currentColor" />
        <path
          d="M 16,62 C 26,52 44,32 62,18 C 72,10 80,8 84,14 C 86,18 82,24 76,28 C 60,40 40,56 28,66 C 22,70 18,68 16,64 Z"
          fill="currentColor"
        />
        <path
          d="M 16,62 C 26,72 44,92 62,106 C 72,114 80,116 84,110 C 86,106 82,100 76,96 C 60,84 40,68 28,58 C 22,54 18,56 16,60 Z"
          fill="currentColor"
        />
        <circle cx="18" cy="62" r="10" fill="hsl(var(--ember))" />
      </g>
    </svg>
  );
}

interface KnowMeWordmarkProps {
  className?: string;
}

export function KnowMeWordmark({ className }: KnowMeWordmarkProps) {
  return (
    <span className={className} style={{ fontFamily: "'Space Grotesk', sans-serif", fontWeight: 700, letterSpacing: "-0.025em" }}>
      Know<span style={{ color: "hsl(var(--ember))" }}>Me</span>
    </span>
  );
}
