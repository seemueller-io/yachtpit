import * as React from 'react';

interface VesselMarkerProps {
    size?: number;
    color?: string;
    heading?: number;
}

const vesselStyle = {
    cursor: 'pointer',
    stroke: '#fff',
    strokeWidth: 2
};

function VesselMarker({ size = 12, color = '#0066cc', heading = 0 }: VesselMarkerProps) {
    return (
        <svg 
            height={size} 
            width={size} 
            viewBox="0 0 24 24" 
            style={{
                ...vesselStyle,
                transform: `rotate(${heading}deg)`,
                transformOrigin: 'center'
            }}
        >
            <circle 
                cx="12" 
                cy="12" 
                r="10" 
                fill={color}
            />
            {/* Small arrow to indicate heading */}
            <path 
                d="M12 4 L16 12 L12 10 L8 12 Z" 
                fill="#fff"
            />
        </svg>
    );
}

export default React.memo(VesselMarker);