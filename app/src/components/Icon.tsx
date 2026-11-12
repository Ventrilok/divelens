// Line icons (same set as the mockups).

const paths = {
  logo: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M5.5 13.5c1.3 0 1.3-1.6 2.6-1.6s1.3 1.6 2.6 1.6 1.3-1.6 2.6-1.6 1.3 1.6 2.6 1.6 1.3-1.6 2.6-1.6" />
    </>
  ),
  list: <path d="M8 6h13M8 12h13M8 18h13M3.5 6h.01M3.5 12h.01M3.5 18h.01" />,
  folder: <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />,
  tag: (
    <>
      <path d="M20.6 13.4l-7.2 7.2a2 2 0 0 1-2.8 0L3 13V3h10l7.6 7.6a2 2 0 0 1 0 2.8z" />
      <circle cx="7.5" cy="7.5" r="1.5" />
    </>
  ),
  settings: (
    <>
      <path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0" />
      <circle cx="16" cy="6" r="2" />
      <circle cx="10" cy="12" r="2" />
      <circle cx="18" cy="18" r="2" />
    </>
  ),
  search: (
    <>
      <circle cx="11" cy="11" r="7" />
      <path d="M21 21l-4.3-4.3" />
    </>
  ),
  chevron: <path d="M6 9l6 6 6-6" />,
  close: <path d="M18 6L6 18M6 6l12 12" />,
  curve: <path d="M3 6c3 0 3 9 6 9s3-5 6-5 3 7 6 7" />,
  noCurve: <path d="M3 12h4M10 12h4M17 12h4" />,
  check: <path d="M20 6L9 17l-5-5" />,
  arrow: <path d="M5 12h14M13 6l6 6-6 6" />,
  warn: (
    <>
      <path d="M12 3l9.5 17H2.5z" />
      <path d="M12 10v4M12 17.5h.01" />
    </>
  ),
  spark: <path d="M12 3l1.8 4.7L18.5 9.5l-4.7 1.8L12 16l-1.8-4.7L5.5 9.5l4.7-1.8z" />,
  database: (
    <>
      <ellipse cx="12" cy="6" rx="7" ry="3" />
      <path d="M5 6v12c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 12c0 1.7 3.1 3 7 3s7-1.3 7-3" />
    </>
  ),
};

export type IconName = keyof typeof paths;

export function Icon({ name, size = 18, className, label }: { name: IconName; size?: number; className?: string; label?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      className={`ic ${className ?? ""}`}
      style={{ width: size, height: size }}
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
    >
      {paths[name]}
    </svg>
  );
}
