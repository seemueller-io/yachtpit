import type {VesselStatus} from "@/types.ts";
import {Box} from "@chakra-ui/react";
import {getNeumorphicStyle} from "@/theme/neumorphic-theme.ts";

export function GpsFeed(props: { vesselPosition: VesselStatus, colorMode: 'light' | 'dark' }) {
    return <>
        <Box
            position="relative"
            zIndex={1}
            fontSize="sm"
            maxH="20%"
            fontFamily="monospace"
            backdropFilter="blur(10px)"
            {...getNeumorphicStyle(props.colorMode)}
        >
            <Box fontWeight="bold" mb={3} fontSize="md">GPS Feed</Box>
            <Box mb={1}>Lat: {props.vesselPosition.latitude.toFixed(6)}°</Box>
            <Box mb={1}>Lon: {props.vesselPosition.longitude.toFixed(6)}°</Box>
            <Box mb={1}>Heading: {props.vesselPosition.heading.toFixed(1)}°</Box>
            <Box>Speed: {props.vesselPosition.speed.toFixed(1)} kts</Box>
        </Box>
    </>;
}