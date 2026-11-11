import { STAGES } from "@/lib/tree";

// One drawing per stage: seed, sprout, sapling, young tree, tree. Pure: the stage comes from
// the balance (lib/tree.ts). Each layer fades in as the tree grows.
// The drawing fills its container's width; the page sets that width.
export function Tree({ stage }: { stage: number }) {
  const show = (n: number) => ({
    opacity: stage >= n ? 1 : 0,
    transition: "opacity 600ms ease",
  });
  return (
    <svg
      viewBox="0 0 200 200"
      role="img"
      aria-label={`${STAGES[stage].name}, stage ${stage + 1} of 5`}
      data-testid="tree"
      data-stage={stage}
      className="block w-full h-auto"
    >
      <ellipse cx="100" cy="178" rx="70" ry="12" fill="var(--color-soil)" opacity="0.45" />
      <path d="M40 178 Q100 160 160 178 Z" fill="var(--color-soil)" />
      {/* Seed: only before the sprout */}
      <ellipse cx="100" cy="168" rx="9" ry="6" fill="var(--color-bark)" style={{ opacity: stage === 0 ? 1 : 0, transition: "opacity 600ms" }} />
      {/* Sprout */}
      <g style={show(1)}>
        <path d="M100 170 L100 140" stroke="var(--color-leaf)" strokeWidth="4" strokeLinecap="round" />
        <path d="M100 148 Q84 136 80 124 Q96 126 100 144" fill="var(--color-leaf)" />
        <path d="M100 144 Q114 128 124 126 Q120 140 100 150" fill="var(--color-leaf)" />
      </g>
      {/* Sapling */}
      <g style={show(2)}>
        <path d="M100 170 L100 104" stroke="var(--color-bark)" strokeWidth="6" strokeLinecap="round" />
        <circle cx="100" cy="100" r="22" fill="var(--color-leaf)" />
        <circle cx="84" cy="112" r="14" fill="var(--color-leaf)" />
        <circle cx="116" cy="112" r="14" fill="var(--color-leaf)" />
      </g>
      {/* Young tree */}
      <g style={show(3)}>
        <path d="M100 172 L100 84" stroke="var(--color-bark)" strokeWidth="10" strokeLinecap="round" />
        <circle cx="100" cy="74" r="38" fill="var(--color-leaf)" />
        <circle cx="70" cy="96" r="24" fill="var(--color-leaf-dark)" />
        <circle cx="130" cy="96" r="24" fill="var(--color-leaf-dark)" />
      </g>
      {/* Tree with fruit */}
      <g style={show(4)}>
        <circle cx="100" cy="58" r="46" fill="var(--color-leaf)" />
        <circle cx="62" cy="84" r="30" fill="var(--color-leaf-dark)" />
        <circle cx="138" cy="84" r="30" fill="var(--color-leaf-dark)" />
        <circle cx="84" cy="52" r="5" fill="#d9483b" />
        <circle cx="120" cy="66" r="5" fill="#d9483b" />
        <circle cx="70" cy="88" r="5" fill="#d9483b" />
        <circle cx="132" cy="94" r="5" fill="#d9483b" />
      </g>
    </svg>
  );
}
