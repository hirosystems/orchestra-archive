import styled from "styled-components";
import { useRootSelector } from "../../hooks/useRootSelector";
import { selectIsNetworkHealthy } from "../../states/NetworkingState";
import { Blocks } from './Blocks';
import { StartControl } from './StartControl';
import { ChainInfo } from './ChainInfo';
import { DebugControl } from './DebugControl';
import { ToggleControl } from "./ToggleControl";

export const Container = styled.div`
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    padding-left: 16px;
    padding-right: 16px;
    padding-top: 12px;
`

export const ChainControls = styled.div`
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
`

const ChainView = () => {

    const isNetworkHealthy = useRootSelector(selectIsNetworkHealthy);

    let control = <StartControl />;
    if (isNetworkHealthy) {
        control = <ToggleControl />;
    }

    return (
        <Container>
            <ChainControls>
                {control}
                <DebugControl />
            </ChainControls>
            <Blocks />
            <ChainInfo />
        </Container>
    );
};

export { ChainView };

// let content = undefined;
// if (!networkBooted) {
//     <div onClick={handleBootNetwork}>
// }