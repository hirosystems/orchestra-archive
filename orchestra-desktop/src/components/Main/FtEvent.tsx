import { Timeline, StyledOcticon } from '@primer/react'
import { PaperAirplaneIcon, RocketIcon, FlameIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { FtBurnEvent, FtMintEvent, FtTransferEvent } from '../../states/NetworkingState';

export const Container = styled.div`
    color: rgba(255, 255, 255, 0.8);
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const FtEvent = (props: { event: FtMintEvent | FtTransferEvent | FtBurnEvent }) => {
    let icon = RocketIcon;
    let color = 'success.emphasis';
    let label = '';
    if ('Burn' in props.event) {
        icon = FlameIcon;
        color = 'danger.emphasis';
        label = `${props.event.Burn.amount} tokens were burnt by ${props.event.Burn.sender}`
    } else if ('Transfer' in props.event) {
        color = 'accent.emphasis';
        icon = PaperAirplaneIcon;
        label = `${props.event.Transfer.amount} tokens were transfered from ${props.event.Transfer.sender} to ${props.event.Transfer.recipient}`
    } else if ('Mint' in props.event) {
        label = `${props.event.Mint.amount} tokens were minted for ${props.event.Mint.recipient}`
    }

    return (
        <Timeline.Item>
            <Timeline.Badge sx={{ bg: color }}>
                <StyledOcticon icon={icon} sx={{ color: 'fg.onEmphasis' }} />
            </Timeline.Badge>
            <Timeline.Body sx={{ color: 'fg.onEmphasis' }}>{label}</Timeline.Body>
        </Timeline.Item>
    );
};

export { FtEvent };