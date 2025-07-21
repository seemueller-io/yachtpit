import 'mapbox-gl/dist/mapbox-gl.css';
import {Button, Menu, Portal} from '@chakra-ui/react';
import {useColorMode} from './components/ui/color-mode';
import {useState} from "react";
import {getNeumorphicColors, getNeumorphicStyle} from './theme/neumorphic-theme';

export const layers = [
    { name: 'OSM', value: 'mapbox://styles/mapbox/dark-v11' },
    { name: 'Satellite', value: 'mapbox://styles/mapbox/satellite-v9' },
];



// const vesselLayerStyle: CircleLayerSpecification = {
//     id: 'vessel',
//     type: 'circle',
//     paint: {
//         'circle-radius': 8,
//         'circle-color': '#ff4444',
//         'circle-stroke-width': 2,
//         'circle-stroke-color': '#ffffff'
//     },
//     source: ''
// };


export type Layer = { name: string; value: string };
export type Layers = Layer[];

// interface MapViewParams {
//     latitude: number;
//     longitude: number;
//     zoom: number;
// }

// interface AuthParams {
//     authenticated: boolean;
//     token: string | null;
// }

export function LayerSelector(props: { onClick: (layer: Layer) => Promise<void> }) {
    const { colorMode } = useColorMode();
    const [selectedLayer, setSelectedLayer] = useState(layers[0]);
    const neumorphicStyle = getNeumorphicStyle(colorMode as 'light' | 'dark');
    const colors = getNeumorphicColors(colorMode as 'light' | 'dark');

    return (
        <Menu.Root>
            <Menu.Trigger asChild>
                <Button
                    size="sm"
                    variant="surface"
                    {...neumorphicStyle}
                >
                    {selectedLayer?.name || 'Layer'}
                </Button>
            </Menu.Trigger>
            <Portal>
                <Menu.Positioner>
                    <Menu.Content
                        minW="200px"
                        py={2}
                        {...neumorphicStyle}
                    >
                        {layers.map(layer => (
                            <Menu.Item
                                key={layer.value}
                                id={layer.value}
                                value={layer.value}
                                borderRadius={6}
                                transition="all 0.2s ease-in-out"
                                _hover={{
                                    bg: colors.accent + '20',
                                    transform: 'translateY(-1px)',
                                }}
                                onClick={(e) => {
                                    // @ts-ignore
                                    console.log(e.target.id)
                                    setSelectedLayer(layer);
                                    props.onClick(layer);
                                }}
                            >
                                {layer.name}
                            </Menu.Item>
                        ))}
                    </Menu.Content>
                </Menu.Positioner>
            </Portal>
        </Menu.Root>
    );
}
