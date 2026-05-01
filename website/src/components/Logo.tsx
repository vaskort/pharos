function LogomarkPaths() {
  return (
    <>
      {/* Tower */}
      <path d="M14 5 L22 5 L26 28 L10 28 Z" fill="currentColor" />
      {/* Lantern */}
      <rect x="14.5" y="8.5" width="7" height="3" fill="#0f1f1b" />
      {/* Beam */}
      <path d="M22 10 L34 6 L34 14 Z" fill="#f5b64a" />
    </>
  )
}

export function Logomark(props: React.ComponentPropsWithoutRef<'svg'>) {
  return (
    <svg aria-hidden="true" viewBox="0 0 36 36" fill="none" {...props}>
      <LogomarkPaths />
    </svg>
  )
}

export function Logo(props: React.ComponentPropsWithoutRef<'svg'>) {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 140 36"
      fill="none"
      {...props}
    >
      <LogomarkPaths />
      <text
        x="46"
        y="25"
        fontFamily="var(--font-display, ui-sans-serif, system-ui)"
        fontSize="20"
        fontWeight="600"
        fill="currentColor"
        letterSpacing="-0.01em"
      >
        Pharos
      </text>
    </svg>
  )
}
