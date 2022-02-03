import { Text, Timeline, StyledOcticon, Link } from '@primer/react'
import {PaperAirplaneIcon, RocketIcon, FlameIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { FtBurnEvent, FtMintEvent, FtTransferEvent } from '../../states/NetworkingState';

export const Container = styled.div`
    color: rgb(55, 53, 47);
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const FtEvent = (props: { event: FtMintEvent|FtTransferEvent|FtBurnEvent }) => {
    let icon = RocketIcon;
    let color = 'success.emphasis';
    let label = '';
    if ('FTBurnEvent' in props.event) {
        icon = FlameIcon;
        color = 'danger.emphasis';
        label = `${props.event.FTBurnEvent.amount} tokens were burnt by ${props.event.FTBurnEvent.sender}`
    } else if ('FTTransferEvent' in props.event) {
        color = 'accent.emphasis';
        icon = PaperAirplaneIcon;
        label = `${props.event.FTTransferEvent.amount} tokens were transfered from ${props.event.FTTransferEvent.sender} to ${props.event.FTTransferEvent.recipient}`
    } else if ('FTMintEvent' in props.event) {
        label = `${props.event.FTMintEvent.amount} tokens were minted for ${props.event.FTMintEvent.recipient}`
    }

    return (
        <Timeline.Item>
            <Timeline.Badge sx={{bg: color}}>
                <StyledOcticon icon={icon} sx={{color: 'fg.onEmphasis'}} />
            </Timeline.Badge>
            <Timeline.Body>{label}</Timeline.Body>
        </Timeline.Item>
    );
};

export { FtEvent };