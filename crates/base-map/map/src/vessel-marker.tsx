import React from 'react';

interface VesselMarkerProps {
    heading: number;
    color?: string;
    size?: number;
}

const VesselMarker: React.FC<VesselMarkerProps> = ({ 
    heading, 
    color = '#0066cc', 
    size = 16 
}) => {
    return (
        <div
            style={{
                width: size,
                height: size,
                transform: `rotate(${heading}deg)`,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center'
            }}
        >
            <svg
                width={size}
                height={size}
                viewBox="0 0 24 24"
                fill={color}
                style={{
                    filter: 'drop-shadow(0 2px 4px rgba(0,0,0,0.3))'
                }}
            >
                {/* Simple vessel shape - triangle pointing up (north) */}
                <path d="M12 2 L20 20 L12 16 L4 20 Z" />
            </svg>
        </div>
    );
};

export default VesselMarker;