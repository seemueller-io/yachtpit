import type {VesselData} from "@/ais-provider.tsx";
import type { VesselStatus } from "@/types";
import { Box } from "@chakra-ui/react";
import {getNeumorphicStyle} from "@/theme/neumorphic-theme.ts";

export function AisFeed(props: {
    vesselPosition: VesselStatus | null,
    colorMode: "light" | "dark",
    connectionStatus: string,
    vesselData: VesselData[],
    aisError: string | null,
    aisConnected: boolean
}) {
    return <Box
        position="relative"
        zIndex={1}
        p={4}
        fontSize="sm"
        fontFamily="monospace"
        maxH="20%"
        backdropFilter="blur(10px)"
        {...getNeumorphicStyle(props.colorMode as "light" | "dark")}
    >
        <Box fontWeight="bold" mb={3} fontSize="md">AIS Status</Box>
        <Box mb={1}>Status: {props.connectionStatus}</Box>
        <Box mb={1}>Vessels: {props.vesselData.length}</Box>
        {props.aisError && <Box color="red.500" fontSize="xs">Error: {props.aisError}</Box>}
        {props.aisConnected && (
            <Box color="green.500" fontSize="xs" mt={2}>
                ✓ Connected to AIS server
            </Box>
        )}
    </Box>;
}
