import { cn } from "@/lib/utils";

interface UarLogoProps {
  size?: number;
  className?: string;
  label?: string;
  decorative?: boolean;
}

export function UarLogo({
  size = 28,
  className,
  label = "Universal Agent Runtime",
  decorative = true,
}: UarLogoProps) {
  return (
    <svg
      viewBox="0 0 96 96"
      width={size}
      height={size}
      fill="none"
      className={className}
      aria-hidden={decorative || undefined}
      aria-label={decorative ? undefined : label}
      role={decorative ? undefined : "img"}
    >
      <path d="M40 22 24 74" stroke="currentColor" strokeWidth="9" strokeLinecap="round" />
      <path d="M68 22 52 74" stroke="currentColor" strokeWidth="9" strokeLinecap="round" />
      <circle cx="70" cy="68" r="6" fill="currentColor" />
    </svg>
  );
}

interface UarBrandAssetProps {
  className?: string;
  decorative?: boolean;
}

export function UarWordmark({ className, decorative = false }: UarBrandAssetProps) {
  return (
    <span
      className={cn("block", className)}
      aria-hidden={decorative || undefined}
      aria-label={decorative ? undefined : "Universal Agent Runtime"}
      role={decorative ? undefined : "img"}
    >
      <svg aria-hidden="true" viewBox="0 0 520 96" className="block h-full w-full" fill="none">
        <g transform="translate(0,6)">
          <path
            d="M40 22 L24 74"
            className="stroke-[#E04E28] dark:stroke-[#FF6A3D] [.high-contrast_&]:stroke-[#FF6A3D]"
            strokeWidth="9"
            strokeLinecap="round"
          />
          <path
            d="M68 22 L52 74"
            className="stroke-[#0B0F14] dark:stroke-[#E04E28] [.high-contrast_&]:stroke-[#E04E28]"
            strokeWidth="9"
            strokeLinecap="round"
          />
          <circle
            cx="70"
            cy="68"
            r="6"
            className="fill-[#0B0F14] dark:fill-[#E8EDF3] [.high-contrast_&]:fill-[#E8EDF3]"
          />
        </g>
        <text
          x="112"
          y="46"
          className="fill-[#0B0F14] font-display text-[30px] font-bold tracking-[-0.9px] dark:fill-[#E8EDF3] [.high-contrast_&]:fill-[#E8EDF3]"
        >
          Universal Agent Runtime
        </text>
        <text
          x="112"
          y="70"
          className="fill-[#E04E28] font-mono text-[13px] font-medium tracking-[2.6px] dark:fill-[#FF6A3D] [.high-contrast_&]:fill-[#FF6A3D]"
        >
          // UAR RUNTIME
        </text>
      </svg>
    </span>
  );
}

export function UarAppIcon({ className, decorative = false }: UarBrandAssetProps) {
  return (
    <span
      className={cn("block", className)}
      aria-hidden={decorative || undefined}
      aria-label={decorative ? undefined : "Universal Agent Runtime"}
      role={decorative ? undefined : "img"}
    >
      <svg aria-hidden="true" viewBox="0 0 96 96" className="block h-full w-full" fill="none">
        <rect
          width="96"
          height="96"
          rx="24"
          className="fill-[#F7F7F8] dark:fill-[#0B0F14] [.high-contrast_&]:fill-[#0B0F14]"
        />
        <path
          d="M40 22 L24 74"
          className="stroke-[#E04E28] dark:stroke-[#FF6A3D] [.high-contrast_&]:stroke-[#FF6A3D]"
          strokeWidth="9"
          strokeLinecap="round"
        />
        <path
          d="M68 22 L52 74"
          className="stroke-[#0B0F14] dark:stroke-[#E04E28] [.high-contrast_&]:stroke-[#E04E28]"
          strokeWidth="9"
          strokeLinecap="round"
        />
        <circle
          cx="70"
          cy="68"
          r="6"
          className="fill-[#0B0F14] dark:fill-[#E8EDF3] [.high-contrast_&]:fill-[#E8EDF3]"
        />
      </svg>
    </span>
  );
}
