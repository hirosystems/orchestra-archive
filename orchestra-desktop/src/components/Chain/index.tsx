
import styled from "styled-components";
import { useRootSelector, useRootDispatch } from "../../hooks/useRootSelector";
import { getBitcoinChainTip, getStacksChainTip } from "../../states/BlocksExplorerState";
import { selectNetworkBooted, bootNetwork } from "../../states/NetworkingState";
import { Block } from './Block';
import { StyledOcticon } from "@primer/react";
import { ZapIcon, PlayIcon } from "@primer/octicons-react";
import { MouseEvent } from 'react';

export const ChainOverview = styled.div`
padding-top: 18px;
padding-bottom: 12px;
padding-right: 12px;
height: 100%;
cursor: default;
`

export const ChainBackground = styled.div`
background-color: rgba(0, 0, 0, 0.8); // rgb(247, 246, 243);
height: 100%;
padding-right: 16px;
padding-left: 16px;
border-radius: 8px;
display: flex;
flex-flow: row wrap;
min-width: 744px;
`

export const Blocks = styled.div`
display: flex;
flex-flow: row wrap;
gap: 8px;
cursor: default;
`

export const ChainControl = styled.div`
width: 80px;
height: 64px;
`

export const ChainBar = styled.div`

`

export const ChainTopControls = styled.div`
display: flex;
flex-flow: row wrap;
padding-top: 5px;
padding-bottom: 4px;
justify-content: space-between;
`

export const ChainLeftInfo = styled.div`
`

export const StartNetwork = styled.div`
width: 100%;
height: 100%;
display: flex;
justify-content: center;
padding-top: 8px;
`

export const ChainCenterInfo = styled.div`
min-width: 100px;
flex: 'flex-grow';
`

export const ChainRightInfo = styled.div`
justify-content: 'flex-end';
`

export const ChainPicker = styled.div`
    text-transform: uppercase;
    font-size: 10px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    color: ${(props: { isFieldActive: boolean }) => props.isFieldActive ? "rgb(255, 255, 255)" : "rgba(255, 255, 255, 0.4)"};
    &:hover {
        color: rgb(255, 255, 255);
    }
`

const Chain = () => {
    
    const networkBooted = useRootSelector(selectNetworkBooted);
    const stacksChainTip = useRootSelector(getStacksChainTip);
    let dispatch = useRootDispatch();

    function handleBootNetwork(event: MouseEvent) {
        event.preventDefault();
        dispatch(bootNetwork());
    }

    let content = undefined;
    if (!networkBooted) {
        content = (
            <ChainBackground data-tauri-drag-region>
                <StartNetwork >
                    <div onClick={handleBootNetwork}>
                        <StyledOcticon icon={PlayIcon} size={48} sx={{mr: 2, color: 'white'}} />
                    </div>
                </StartNetwork>
            </ChainBackground>
        )
    } else {
        let blocks = [];

        let knownChainTipHeight = stacksChainTip ? stacksChainTip.metadata.pox_cycle_position : 0;
        for (let i = 0; i < 10; i++) {
            let isKnown = i <= knownChainTipHeight;
            blocks.push(
                <Block key={i} blockHeight={i} isKnown={isKnown}/>
            )
        }
    
        let poxCycle = stacksChainTip ? stacksChainTip.metadata.pox_cycle_index : 0;    
        content = (
            <ChainBackground data-tauri-drag-region>
                <ChainBar>
                    <ChainTopControls>
                        <ChainLeftInfo>
                            <ChainPicker isFieldActive={true}>STACKS DEVNET</ChainPicker>
                            <ChainPicker isFieldActive={false}>BITCOIN REGTEST</ChainPicker>
                        </ChainLeftInfo>
                        <ChainCenterInfo></ChainCenterInfo>
                        <ChainRightInfo>
                        <ChainPicker isFieldActive={true}>POX CYCLE #{poxCycle}</ChainPicker>
                        </ChainRightInfo>
                    </ChainTopControls>
                    <Blocks>
                        {blocks}
                    </Blocks>
                </ChainBar>
            </ChainBackground>
        )
    }

    return (
        <ChainOverview data-tauri-drag-region>
            {content}
        </ChainOverview>);
};

export { Chain };