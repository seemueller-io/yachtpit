import {Box} from "@chakra-ui/react";

interface SearchResultProps {
    onKeyPress: (e: any) => Promise<void>;
    colors: {
        bg: string;
        surface: string;
        text: string;
        textSecondary: string;
        accent: string;
        shadow: { dark: string; light: string }
    } | {
        bg: string;
        surface: string;
        text: string;
        textSecondary: string;
        accent: string;
        shadow: { dark: string; light: string }
    };
    onClick: () => Promise<void>;
    result: any;
}

export function SearchResult(props: SearchResultProps) {
    return <Box
        p={3}
        cursor="pointer"
        borderRadius="8px"
        transition="all 0.2s ease-in-out"
        onKeyPress={props.onKeyPress}
        _hover={{
            bg: props.colors.accent + "20",
            transform: "translateY(-1px)",
        }}
        onClick={props.onClick}
    >
        {`${props.result.lat}, ${props.result.lon}`}
    </Box>;
}
