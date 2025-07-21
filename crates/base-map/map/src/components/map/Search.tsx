import {Box, Button, Input, Text} from "@chakra-ui/react";
import {getNeumorphicStyle} from "@/theme/neumorphic-theme.ts";

export function Search(props: {
    onClick: () => Promise<void>,
    colorMode: "light" | "dark",
    searchOpen: boolean,
    onKeyDown: (e: any) => void,
    value: string,
    onChange: (e: any) => void,
    onKeyPress: (e: any) => Promise<void>,
    searchResults: any[],
    callbackfn: (result: any, index: any) => any // JSX.Element
}) {
    return <Box
        display="flex"
        alignItems="center"
        position="relative"
    >
        <Button
            size="sm"
            variant="surface"
            onClick={props.onClick}
            mr={2}
            {...getNeumorphicStyle(props.colorMode as "light" | "dark")}
        >
            <Text>Search...</Text>
        </Button>
        {props.searchOpen && <Box
            w="200px"
            transform={`translateX(${props.searchOpen ? "0" : "100%"})`}
            opacity={props.searchOpen ? 1 : 0}
            onKeyDown={props.onKeyDown}
            backdropFilter="blur(10px)"
            {...getNeumorphicStyle(props.colorMode as "light" | "dark", "pressed")}
        >
            <Input
                placeholder="Search..."
                size="sm"
                value={props.value}
                onChange={props.onChange}
                onKeyPress={props.onKeyPress}
                border="none"
                {...getNeumorphicStyle(props.colorMode as "light" | "dark", "pressed")}
            />
            {props.searchResults.length > 0 && (
                <Box
                    position="absolute"
                    top="100%"
                    left={0}
                    w="200px"
                    zIndex={2}
                    mt={2}

                    backdropFilter="blur(10px)"
                    {...getNeumorphicStyle(props.colorMode as "light" | "dark")}
                >
                    {props.searchResults.map(props.callbackfn)}
                </Box>
            )}
        </Box>}
    </Box>;
}
