import * as React from 'react';

/**
 * UserLocationMarker
 *  • size  – overall diameter in px (default 24)
 *  • color – dot / ring colour   (default #1E90FF  ⟵  system‑blue)
 *  • pulse – adds a subtle accuracy‑halo animation when true
 */
function UserLocationMarker({
                                size = 24,
                                color = '#1E90FF',
                                pulse = false
                            }) {
    // stroke width scales with size so the ring stays proportionate
    const strokeWidth = size * 0.083;            // ≈ 2px when size = 24

    // keyframes are injected once per page‑load if pulse is ever enabled
    React.useEffect(() => {
        if (!pulse || document.getElementById('ulm‑pulse‑kf')) return;
        const styleTag = document.createElement('style');
        styleTag.id = 'ulm‑pulse‑kf';
        styleTag.textContent = `
      @keyframes ulm‑pulse {
        0%   { r: 0;    opacity: .6; }
        70%  { r: 12px; opacity: 0;  }
        100% { r: 12px; opacity: 0;  }
      }`;
        document.head.appendChild(styleTag);
    }, [pulse]);

    return (
        <svg
            height={size}
            width={size}
            viewBox="0 0 24 24"
            style={{
                display: 'block',
                transform: 'translate(-50%, -50%)', // center on map coordinate
                pointerEvents: 'none'               // let clicks pass through
            }}
        >
            {/* accuracy halo (animated when pulse=true) */}
            {pulse && (
                <circle
                    cx="12"
                    cy="12"
                    r="0"
                    fill={color}
                    opacity=".6"
                    style={{
                        animation: 'ulm‑pulse 2s ease-out infinite'
                    }}
                />
            )}

            {/* outer ring */}
            <circle
                cx="12"
                cy="12"
                r={size / 2 - strokeWidth}
                fill="none"
                stroke={color}
                strokeWidth={strokeWidth}
            />

            {/* inner dot */}
            <circle
                cx="12"
                cy="12"
                r={size * 0.25}   /* ≈ 6px when size = 24 */
                fill={color}
            />
        </svg>
    );
}

export default React.memo(UserLocationMarker);
