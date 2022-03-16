
import styled from "styled-components";
import { useRootSelector } from "../../hooks/useRootSelector";
import { getStacksChainTip } from "../../states/BlocksExplorerState";
import { selectNetworkPaused } from "../../states/NetworkingState";
import { KnownBlock } from './KnownBlock';
import { UnknownBlock } from './UnknownBlock';
import { TipBlock } from './TipBlock';
import { NextBlock } from './NextBlock';

export const Container = styled.div`
display: flex;
flex-flow: row;
gap: 12px;
cursor: default;
justify-content: center;
`

const Blocks = () => {

    const stacksChainTip = useRootSelector(getStacksChainTip);
    const networkPaused = useRootSelector(selectNetworkPaused);

    let blocks = [];

    if (stacksChainTip) {
        let firstBlock = stacksChainTip.block_identifier.index - (stacksChainTip.block_identifier.index % stacksChainTip.metadata.pox_cycle_length);
        let lastBlock = firstBlock + stacksChainTip.metadata.pox_cycle_length;
        let knownChainTipHeight = stacksChainTip.block_identifier.index;
        for (let i = firstBlock; i < lastBlock; i++) {
            if (i < knownChainTipHeight) {
                blocks.push(
                    <KnownBlock key={i} blockHeight={i} />
                )
            } else if (i === knownChainTipHeight) {
                blocks.push(
                    <TipBlock key={i} blockHeight={i} />
                )
            } else if (i === knownChainTipHeight + 1 && !networkPaused) {
                blocks.push(
                    <NextBlock key={i} blockHeight={i} />
                )
            } else {
                blocks.push(
                    <UnknownBlock key={i} blockHeight={i} />
                )
            }
        }
    } else {
        for (let i = 0; i < 10; i++) {
            blocks.push(
                <UnknownBlock key={i} blockHeight={i} />
            )
        }
    }

    return (
        <Container>
            {blocks}
        </Container>
    );
};

export { Blocks };