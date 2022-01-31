
import styled from "styled-components";
import { useRootSelector } from "../../hooks/useRootSelector";
import { getBitcoinChainTip, getStacksChainTip } from "../../states/BlocksExplorerState";
import { Block } from './Block';

export const ChainOverview = styled.div`
padding-top: 18px;
padding-bottom: 12px;
padding-right: 12px;
height: 100%;
cursor: default;
`

export const ChainBackground = styled.div`
background-color: rgb(247, 246, 243);
height: 100%;
padding-right: 8px;
border-radius: 4px;
display: flex;
flex-flow: row wrap;
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
    color: ${(props: { isFieldActive: boolean }) => props.isFieldActive ? "rgb(55, 53, 47)" : "rgb(180, 180, 180)"};
    &:hover {
        color: rgb(55, 53, 47);
    }
`

const Chain = () => {

    const stacksChainTip = useRootSelector(getStacksChainTip);
    let blocks = [];

    let knownChainTipHeight = stacksChainTip ? stacksChainTip.metadata.pox_cycle_position : 0;
    for (let i = 0; i < 10; i++) {
        let isKnown = i <= knownChainTipHeight;
        blocks.push(
            <Block key={i} blockHeight={i} isKnown={isKnown}/>
        )
    }

    let poxCycle = stacksChainTip ? stacksChainTip.metadata.pox_cycle_index : 0;

    return (
        <ChainOverview data-tauri-drag-region>
            {/* <HiroIcon/> */}
            <ChainBackground data-tauri-drag-region>
                <ChainControl></ChainControl>
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
        </ChainOverview>);
};

export { Chain };